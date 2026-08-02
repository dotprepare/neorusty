use crate::config::ServerConfig;
use crate::lifecycle::{
    ServerStartedEvent, ServerStartingEvent, ServerStoppingEvent, ServerTickEvent,
};
use neorusty_core::event::EventBus;
use pumpkin::data::VanillaData;
use pumpkin::PumpkinServer;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct Server {
    pub config: ServerConfig,
    pub event_bus: Arc<EventBus>,
    pub tick_count: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    pumpkin: Option<Arc<PumpkinServer>>,
    server_task: Option<JoinHandle<()>>,
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
            server_task: None,
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

        let basic = build_basic_config(&self.config);
        let advanced = build_advanced_config(&self.config);
        let vanilla_data = VanillaData::load();

        if pumpkin::LOGGER_IMPL.get().is_none() {
            pumpkin::init_logger(&advanced);
        }

        let engine = Arc::new(PumpkinServer::new(basic, advanced, vanilla_data).await);
        let _ = engine.init_plugins().await;

        self.start_tick_task(&engine);

        let server = engine.clone();
        self.server_task = Some(tokio::spawn(async move {
            server.start().await;
        }));

        self.pumpkin = Some(engine);
        self.running.store(true, Ordering::SeqCst);
        let _ = self.event_bus.post(ServerStartedEvent);
    }

    pub async fn stop(&mut self) {
        if !self.running() {
            return;
        }

        let _ = self.event_bus.post(ServerStoppingEvent);
        pumpkin::stop_server();

        if let Some(task) = self.server_task.take() {
            let _ = task.await;
        }
        if let Some(task) = self.tick_task.take() {
            task.abort();
        }

        self.running.store(false, Ordering::SeqCst);
        self.pumpkin = None;
    }

    pub async fn run(&mut self) {
        self.start().await;

        println!(
            "Server running ({} ticks/sec). Press Ctrl+C to stop.",
            self.config.tick_rate_hz
        );

        let signal = shutdown_signal().await;
        println!("Shutdown signal received ({signal}), stopping server...");

        self.stop().await;
    }

    fn start_tick_task(&mut self, engine: &Arc<PumpkinServer>) {
        let bus = self.event_bus.clone();
        let tick_count = self.tick_count.clone();
        let engine = engine.clone();

        self.tick_task = Some(tokio::spawn(async move {
            let mut last = engine.server.tick_count.load(Ordering::Relaxed);
            loop {
                let current = engine.server.tick_count.load(Ordering::Relaxed);
                if current != last {
                    last = current;
                    let tick = current as u64;
                    tick_count.store(tick, Ordering::Relaxed);
                    let _ = bus.post(ServerTickEvent { tick });
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }));
    }

    pub fn player_count(&self) -> usize {
        self.pumpkin.as_ref().map_or(0, |engine| {
            engine
                .server
                .worlds
                .load()
                .iter()
                .map(|world| world.players.load().len())
                .sum()
        })
    }
}

fn build_basic_config(config: &ServerConfig) -> BasicConfiguration {
    BasicConfiguration {
        tps: config.tick_rate_hz as f32,
        default_level_name: "world".to_string(),
        ..Default::default()
    }
}

fn build_advanced_config(config: &ServerConfig) -> AdvancedConfiguration {
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.java.address = config.host;
    advanced.networking.java.max_players = config.max_players;
    advanced.networking.java.motd = config.motd.clone();
    advanced.networking.java.online_mode = config.online_mode;
    advanced.networking.java.encryption = config.online_mode;
    advanced.commands.use_console = false;
    advanced.logging.enabled = true;
    advanced
}

async fn shutdown_signal() -> &'static str {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => "SIGINT",
            _ = terminate.recv() => "SIGTERM",
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        "SIGINT"
    }
}
