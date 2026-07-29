use neorusty_core::event::{EventBus, Event, EventPriority};
use neorusty_server_core::{
    ServerConfig, Server,
    lifecycle::{ServerStartedEvent, ServerTickEvent},
};
use std::sync::{Arc, Mutex};

#[test]
fn test_server_create_default() {
    let bus = Arc::new(EventBus::new());
    let mut server = Server::new(ServerConfig::default(), bus);

    assert!(!server.running);
    assert_eq!(server.player_count(), 0);
    assert!(server.worlds.primary().is_some());
}

#[test]
fn test_server_start_stop() {
    let bus = Arc::new(EventBus::new());
    let mut server = Server::new(ServerConfig::default(), bus.clone());
    assert!(!server.running);

    server.start();
    assert!(server.running);

    server.stop();
    assert!(!server.running);
}

#[test]
fn test_server_tick() {
    let bus = Arc::new(EventBus::new());
    let mut server = Server::new(ServerConfig::default(), bus.clone());
    server.start();

    assert_eq!(server.tick_count, 0);
    server.tick();
    assert_eq!(server.tick_count, 1);
    server.tick();
    assert_eq!(server.tick_count, 2);
}

#[test]
fn test_server_lifecycle_event() {
    let bus = Arc::new(EventBus::new());
    let started = Arc::new(Mutex::new(false));

    let started_clone = started.clone();
    bus.register(EventPriority::Normal, move |_: &ServerStartedEvent| {
        *started_clone.lock().unwrap() = true;
        ServerStartedEvent
    });

    let mut server = Server::new(ServerConfig::default(), bus.clone());
    server.start();

    assert!(*started.lock().unwrap());
}

#[test]
fn test_server_tick_event() {
    let bus = Arc::new(EventBus::new());
    let last_tick = Arc::new(Mutex::new(0u64));

    let last_tick_clone = last_tick.clone();
    bus.register(EventPriority::Normal, move |e: &ServerTickEvent| {
        *last_tick_clone.lock().unwrap() = e.tick;
        ServerTickEvent { tick: e.tick }
    });

    let mut server = Server::new(ServerConfig::default(), bus.clone());
    server.start();
    server.tick();

    assert_eq!(*last_tick.lock().unwrap(), 1);
}

#[test]
fn test_server_add_player() {
    let bus = Arc::new(EventBus::new());
    let mut server = Server::new(ServerConfig::default(), bus);

    let addr = "127.0.0.1:25565".parse().unwrap();
    server.add_player("Alice".to_string(), addr);

    assert_eq!(server.player_count(), 1);
    let player = server.get_player("Alice");
    assert!(player.is_some());
    assert_eq!(player.unwrap().username, "Alice");
}

#[test]
fn test_server_remove_player() {
    let bus = Arc::new(EventBus::new());
    let mut server = Server::new(ServerConfig::default(), bus);

    let addr = "127.0.0.1:25565".parse().unwrap();
    server.add_player("Bob".to_string(), addr);
    assert_eq!(server.player_count(), 1);

    let removed = server.remove_player("Bob");
    assert!(removed.is_some());
    assert_eq!(server.player_count(), 0);
}

#[test]
fn test_world_manager() {
    use neorusty_server_core::world::{World, WorldManager};
    use neorusty_core::resource::ResourceLocation;

    let mut wm = WorldManager::new();
    assert!(wm.primary().is_none());

    let overworld = World::new(
        ResourceLocation::new("minecraft", "overworld"),
        "Overworld".to_string(),
        42,
    );
    wm.add_world(overworld);

    let primary = wm.primary().unwrap();
    assert_eq!(primary.seed, 42);
    assert!(wm.get(&ResourceLocation::new("minecraft", "overworld")).is_some());
}

#[test]
fn test_world_tick() {
    use neorusty_server_core::world::World;
    use neorusty_core::resource::ResourceLocation;

    let mut world = World::new(
        ResourceLocation::new("minecraft", "overworld"),
        "Overworld".to_string(),
        0,
    );
    assert_eq!(world.time, 0);
    world.tick();
    assert_eq!(world.time, 1);
    world.tick();
    assert_eq!(world.time, 2);
}
