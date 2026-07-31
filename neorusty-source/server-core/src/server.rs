use crate::config::ServerConfig;
use crate::lifecycle::{
    ServerStartedEvent, ServerStartingEvent, ServerStoppingEvent, ServerTickEvent,
};
use neorusty_core::event::EventBus;
use pumpkin::data::VanillaData;
use pumpkin::{LOGGER_IMPL, PumpkinServer, stop_server};
use pumpkin_config::logging::LoggingConfig;
use pumpkin_config::networking::NetworkingConfig;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, CommandsConfig, JavaConfig};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
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

        let basic = BasicConfiguration {
            tps: self.config.tick_rate_hz as f32,
            default_level_name: "world".to_string(),
            ..Default::default()
        };
        let advanced = AdvancedConfiguration {
            networking: NetworkingConfig {
                java: JavaConfig {
                    enabled: true,
                    address: self.config.host,
                    online_mode: self.config.online_mode,
                    encryption: self.config.online_mode,
                    max_players: self.config.max_players,
                    motd: self.config.motd.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
            commands: CommandsConfig {
                use_console: false,
                ..Default::default()
            },
            logging: LoggingConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let _ = LOGGER_IMPL.set(None);

        let vanilla_data = VanillaData::load();
        let pumpkin_server = PumpkinServer::new(basic, advanced, vanilla_data).await;
        self.pumpkin = Some(Arc::new(pumpkin_server));

        self.start_tick_task();
        self.running.store(true, Ordering::SeqCst);
        let _ = self.event_bus.post(ServerStartedEvent);

        let server = self.pumpkin.clone().expect("pumpkin server initialized");
        let handle = tokio::spawn(async move {
            server.start().await;
        });
        self.accept_task = Some(handle);
    }

    pub async fn stop(&mut self) {
        if !self.running() {
            return;
        }

        let _ = self.event_bus.post(ServerStoppingEvent);
        self.stop_token.cancel();
        stop_server();

        if let Some(task) = self.accept_task.take() {
            let _ = task.await;
        }
        if let Some(task) = self.tick_task.take() {
            let _ = task.await;
        }

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

    pub fn player_count(&self) -> usize {
        self.pumpkin.as_ref().map_or(0, |server| {
            server
                .server
                .worlds
                .load()
                .iter()
                .map(|world| world.players.load().len())
                .sum()
        })
    }
}
