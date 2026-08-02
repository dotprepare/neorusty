use neorusty_core::event::{EventBus, EventPriority};
use neorusty_server_core::{
    Server, ServerConfig,
    lifecycle::{ServerStartedEvent, ServerTickEvent},
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Mutex;

static SERVER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static TEMP_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

async fn with_temp_dir<T>(fut: impl std::future::Future<Output = T>) -> T {
    let guard = SERVER_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let seq = TEMP_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "neorusty-server-test-{}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::env::set_current_dir(&dir).expect("chdir to temp dir");
    let result = fut.await;
    drop(guard);
    result
}

fn test_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1:0".parse().unwrap(),
        max_players: 20,
        motd: "test".to_string(),
        online_mode: false,
        tick_rate_hz: 20,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_server_create_default() {
    with_temp_dir(async {
        let bus = Arc::new(EventBus::new());
        let server = Server::new(test_config(), bus);

        assert!(!server.running());
        assert_eq!(server.player_count(), 0);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_server_start_stop() {
    with_temp_dir(async {
        let bus = Arc::new(EventBus::new());
        let mut server = Server::new(test_config(), bus);
        assert!(!server.running());

        server.start().await;
        assert!(server.running());

        server.stop().await;
        assert!(!server.running());
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_server_lifecycle_event() {
    with_temp_dir(async {
        let bus = Arc::new(EventBus::new());
        let started = Arc::new(AtomicBool::new(false));

        let started_clone = started.clone();
        bus.register(EventPriority::Normal, move |_: &ServerStartedEvent| {
            started_clone.store(true, Ordering::SeqCst);
            ServerStartedEvent
        });

        let mut server = Server::new(test_config(), bus.clone());
        server.start().await;

        assert!(started.load(Ordering::SeqCst));

        server.stop().await;
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_server_tick_event() {
    with_temp_dir(async {
        let bus = Arc::new(EventBus::new());
        let ticks_seen = Arc::new(AtomicU64::new(0));

        let ticks_clone = ticks_seen.clone();
        bus.register(EventPriority::Normal, move |_: &ServerTickEvent| {
            ticks_clone.fetch_add(1, Ordering::SeqCst);
            ServerTickEvent { tick: 0 }
        });

        let mut server = Server::new(test_config(), bus);
        server.start().await;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            if ticks_seen.load(Ordering::SeqCst) >= 1 {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "no tick event observed before timeout"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(server.tick_count.load(Ordering::Relaxed) >= 1);

        server.stop().await;
    })
    .await;
}
