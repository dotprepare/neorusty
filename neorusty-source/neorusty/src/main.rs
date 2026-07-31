use neorusty_core::event::EventBus;
use neorusty_server_core::{Server, ServerConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = ServerConfig::default();
    let bus = Arc::new(EventBus::new());
    let mut server = Server::new(config, bus);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nShutdown signal received, stopping server...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    println!(
        "Starting NeoRusty server on {}...",
        server.config.host
    );

    server.start().await;
    println!(
        "Server running ({} ticks/sec). Press Ctrl+C to stop.",
        server.config.tick_rate_hz
    );

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    println!("Stopping server...");
    server.stop().await;
    println!("Server stopped.");
}
