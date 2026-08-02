use neorusty_core::event::EventBus;
use neorusty_server_core::{Server, ServerConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = ServerConfig::default();
    let bus = Arc::new(EventBus::new());
    let mut server = Server::new(config, bus);

    println!(
        "Starting NeoRusty server on {}...",
        server.config.host
    );

    server.run().await;

    println!("Server stopped.");
}
