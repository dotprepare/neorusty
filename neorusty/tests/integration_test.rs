use neorusty_core::event::{EventBus, EventPriority};
use neorusty_plugin_system::{Context, Plugin, PluginManager, PluginMetadata};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct PlayerJoinEvent {
    player_name: String,
    cancelled: bool,
}

impl neorusty_core::Event for PlayerJoinEvent {}

#[derive(Debug)]
struct GreeterPlugin;

impl Plugin for GreeterPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "greeter",
            name: "Greeter",
            version: "1.0.0",
            authors: &["neo"],
            description: "Greets joining players",
        }
    }

    fn on_load(&self, ctx: &Context) {
        ctx.on::<PlayerJoinEvent>(EventPriority::Normal, |e| {
            println!("Welcome, {}!", e.player_name);
            PlayerJoinEvent {
                player_name: e.player_name.clone(),
                cancelled: e.cancelled,
            }
        });
    }
}

#[test]
fn test_plugin_event_flow() {
    let bus = Arc::new(EventBus::new());
    let ctx = Context::new("test_plugin".to_string(), bus.clone());

    ctx.on::<PlayerJoinEvent>(EventPriority::Normal, |e: &PlayerJoinEvent| {
        PlayerJoinEvent {
            player_name: format!("formatted_{}", e.player_name),
            cancelled: e.cancelled,
        }
    });

    let event = PlayerJoinEvent {
        player_name: "Alex".to_string(),
        cancelled: false,
    };

    let result = bus.post(event);
    assert_eq!(result.player_name, "formatted_Alex");
}

#[test]
fn test_plugin_manager_flow() {
    let pm = PluginManager::new();
    pm.register(Box::new(GreeterPlugin));

    assert!(pm.is_loaded("greeter"));
    assert_eq!(pm.loaded_plugins().len(), 1);
}

#[test]
fn test_macro_event() {
    use neorusty_macros::Event;

    #[derive(Debug, Clone, Event)]
    struct BlockBreakEvent {
        pos: (i32, i32, i32),
    }

    let bus = EventBus::new();

    bus.register::<BlockBreakEvent>(EventPriority::Normal, |e: &BlockBreakEvent| {
        BlockBreakEvent { pos: (e.pos.0, e.pos.1, e.pos.2) }
    });

    let event = BlockBreakEvent { pos: (10, 64, 20) };
    let result = bus.post(event);
    assert_eq!(result.pos, (10, 64, 20));
}
