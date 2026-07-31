use crate::config::ServerConfig;
use crate::lifecycle::{
    ServerStartedEvent, ServerStartingEvent, ServerStoppingEvent, ServerTickEvent,
};
use neorusty_core::event::EventBus;
use pumpkin::client::Client;
use pumpkin::server::ticker::Ticker;
use pumpkin::server::Server as PumpkinServer;
use pumpkin_config::logging::LoggingConfig;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, BASIC_CONFIG};
use std::fs;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct Server {
    pub config: ServerConfig,
    pub event_bus: Arc<EventBus>,
    pub tick_count: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    pumpkin: Option<Arc<PumpkinServer>>,
    stop_token: CancellationToken,
    accept_task: Option<JoinHandle<()>>,
    tick_task: Option<JoinHandle<()>>,
    pumpkin_tick_task: Option<JoinHandle<()>>,
}

impl Server {
    pub fn new(config: ServerConfig, event_bus: Arc<EventBus>) -> Self {
        Self {
            config,
            event_bus,
            tick_count: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            pumpkin: None,
            stop_token: CancellationToken::new(),
            accept_task: None,
            tick_task: None,
            pumpkin_tick_task: None,
        }
    }

    pub fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn start(&mut self) {
        if self.running() {
            return;
        }

        let _ = self.event_bus.post(ServerStartingEvent);

        write_engine_config(&self.config);
        // Mirror the upstream binary: build rayon's global pool outside of the
        // tokio scope. Ignore the error if it is already initialized.
        let _ = rayon::ThreadPoolBuilder::new().build_global();

        let server = Arc::new(PumpkinServer::new());
        let listener = TcpListener::bind(self.config.host)
            .await
            .expect("Failed to start TcpListener");
        let local_addr = listener.local_addr().expect("listener local address");

        self.pumpkin = Some(server.clone());

        self.start_tick_task();
        self.start_pumpkin_tick_task(&server);
        self.start_accept_task(&server, listener, local_addr);

        self.running.store(true, Ordering::SeqCst);
        let _ = self.event_bus.post(ServerStartedEvent);
    }

    pub async fn stop(&mut self) {
        if !self.running() {
            return;
        }

        let _ = self.event_bus.post(ServerStoppingEvent);
        self.stop_token.cancel();

        if let Some(task) = self.accept_task.take() {
            let _ = task.await;
        }
        if let Some(task) = self.tick_task.take() {
            let _ = task.await;
        }
        // The pumpkin ticker loops forever; detach it (the process exits after
        // stop, mirroring the upstream binary's ctrlc handler).
        self.pumpkin_tick_task = None;

        self.running.store(false, Ordering::SeqCst);
        self.pumpkin = None;
    }

    fn start_tick_task(&mut self) {
        let bus = self.event_bus.clone();
        let tick_count = self.tick_count.clone();
        let token = self.stop_token.clone();
        let tick_rate = self.config.tick_rate_hz.max(1);
        let duration = Duration::from_secs_f64(1.0 / tick_rate as f64);

        self.tick_task = Some(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    _ = tokio::time::sleep(duration) => {
                        let tick = tick_count.fetch_add(1, Ordering::Relaxed);
                        let _ = bus.post(ServerTickEvent { tick });
                    }
                }
            }
        }));
    }

    fn start_pumpkin_tick_task(&mut self, server: &Arc<PumpkinServer>) {
        let server = server.clone();
        let mut ticker = Ticker::new(BASIC_CONFIG.tps);

        self.pumpkin_tick_task = Some(tokio::spawn(async move {
            ticker.run(&server).await;
        }));
    }

    fn start_accept_task(
        &mut self,
        server: &Arc<PumpkinServer>,
        listener: TcpListener,
        local_addr: SocketAddr,
    ) {
        let server = server.clone();
        let token = self.stop_token.clone();

        self.accept_task = Some(tokio::spawn(async move {
            let mut player_count = 0usize;
            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    result = listener.accept() => {
                        let (connection, address) = match result {
                            Ok(pair) => pair,
                            Err(e) => {
                                log::error!("accept error: {e}");
                                continue;
                            }
                        };

                        if let Err(e) = connection.set_nodelay(true) {
                            log::warn!("failed to set TCP_NODELAY {e}");
                        }

                        player_count += 1;
                        let id = player_count;
                        log::info!("Accepted connection from: {address} (id: {id})");

                        let client = Arc::new(Client::new(id, connection, local_addr));
                        let server = server.clone();
                        tokio::spawn(async move {
                            while !client.closed.load(Ordering::Relaxed)
                                && !client.make_player.load(Ordering::Relaxed)
                            {
                                let open = client.poll().await;
                                if open {
                                    client.process_packets(&server).await;
                                }
                            }
                            if client.make_player.load(Ordering::Relaxed) {
                                let id = client.id;
                                log::debug!("Creating player for id {id}");
                                let (player, world) = server.add_player(id, client).await;
                                world.spawn_player(&BASIC_CONFIG, player.clone()).await;
                                while !player.client.closed.load(Ordering::Relaxed) {
                                    let open = player.client.poll().await;
                                    if open {
                                        player.process_packets(&server).await;
                                    }
                                }
                                player.remove().await;
                                server.remove_player().await;
                            }
                        });
                    }
                }
            }
        }));
    }

    pub fn player_count(&self) -> usize {
        self.pumpkin.as_ref().map_or(0, |server| {
            server
                .worlds
                .iter()
                .map(|world| {
                    world
                        .current_players
                        .try_lock()
                        .map_or(0, |players| players.len())
                })
                .sum()
        })
    }
}

fn write_engine_config(config: &ServerConfig) {
    let basic = BasicConfiguration {
        server_address: config.host,
        max_players: config.max_players,
        online_mode: config.online_mode,
        encryption: config.online_mode,
        motd: config.motd.clone(),
        tps: config.tick_rate_hz as f32,
        ..Default::default()
    };
    let toml_str = toml::to_string(&basic).expect("serialize pumpkin configuration");
    fs::write("configuration.toml", toml_str).expect("write pumpkin configuration.toml");

    let advanced = AdvancedConfiguration {
        logging: LoggingConfig {
            enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let toml_str = toml::to_string(&advanced).expect("serialize pumpkin features config");
    fs::write("features.toml", toml_str).expect("write pumpkin features.toml");
}
