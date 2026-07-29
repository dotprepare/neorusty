use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: SocketAddr,
    pub max_players: u32,
    pub motd: String,
    pub online_mode: bool,
    pub tick_rate_hz: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1:25565".parse().unwrap(),
            max_players: 20,
            motd: "NeoRusty Server".to_string(),
            online_mode: false,
            tick_rate_hz: 20,
        }
    }
}
