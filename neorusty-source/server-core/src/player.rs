use std::fmt;
use std::net::SocketAddr;
use std::time::Instant;
use neorusty_core::resource::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid(pub [u8; 16]);

impl Uuid {
    pub fn new_v4() -> Self {
        let mut bytes = [0u8; 16];
        getrandom::getrandom(&mut bytes).expect("rng");
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Self(bytes)
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.0;
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub uuid: Uuid,
    pub username: String,
    pub address: SocketAddr,
    pub game_mode: GameMode,
    pub level: u32,
    pub xp: f32,
    pub health: f32,
    pub max_health: f32,
    pub food: u32,
    pub joined_at: Instant,
    pub dimension: ResourceLocation,
}

impl Player {
    pub fn new(username: String, address: SocketAddr) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            username,
            address,
            game_mode: GameMode::Survival,
            level: 0,
            xp: 0.0,
            health: 20.0,
            max_health: 20.0,
            food: 20,
            joined_at: Instant::now(),
            dimension: ResourceLocation::new("minecraft", "overworld"),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }
}
