use crate::config::ServerConfig;
use crate::lifecycle::{
    ServerStartedEvent, ServerStartingEvent, ServerStoppingEvent, ServerTickEvent,
};
use crate::player::Player;
use crate::world::{World, WorldManager};
use neorusty_core::event::EventBus;
use neorusty_core::resource::ResourceLocation;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Server {
    pub config: ServerConfig,
    pub running: bool,
    pub tick_count: u64,
    pub worlds: WorldManager,
    pub players: HashMap<String, Player>,
    pub event_bus: Arc<EventBus>,
}

impl Server {
    pub fn new(config: ServerConfig, event_bus: Arc<EventBus>) -> Self {
        let mut worlds = WorldManager::new();
        let overworld = World::new(
            ResourceLocation::new("minecraft", "overworld"),
            "Overworld".to_string(),
            0,
        );
        worlds.add_world(overworld);

        Self {
            config,
            running: false,
            tick_count: 0,
            worlds,
            players: HashMap::new(),
            event_bus,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        let _ = self.event_bus.post(ServerStartingEvent);
        let _ = self.event_bus.post(ServerStartedEvent);
    }

    pub fn stop(&mut self) {
        self.running = false;
        let _ = self.event_bus.post(ServerStoppingEvent);
    }

    pub fn tick(&mut self) {
        self.worlds.tick_all();
        self.tick_count += 1;
        let _ = self.event_bus.post(ServerTickEvent {
            tick: self.tick_count,
        });
    }

    pub fn add_player(&mut self, username: String, address: SocketAddr) {
        let player = Player::new(username.clone(), address);
        self.players.insert(username, player);
    }

    pub fn remove_player(&mut self, username: &str) -> Option<Player> {
        self.players.remove(username)
    }

    pub fn get_player(&self, username: &str) -> Option<&Player> {
        self.players.get(username)
    }

    pub fn get_player_mut(&mut self, username: &str) -> Option<&mut Player> {
        self.players.get_mut(username)
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}
