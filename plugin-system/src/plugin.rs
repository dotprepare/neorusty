use crate::loader::{PluginLoader, PluginMetadata};
use dashmap::DashMap;
use neorusty_core::event::{Event, EventBus, EventPriority};
use std::path::Path;
use std::sync::Arc;

pub trait Plugin: Send + Sync + std::fmt::Debug {
    fn metadata(&self) -> PluginMetadata;

    fn on_load(&self, ctx: &Context);

    fn on_unload(&self, _ctx: &Context) {}
}

pub struct Context {
    pub(crate) event_bus: Arc<EventBus>,
    pub(crate) plugin_id: String,
}

impl Context {
    pub fn new(plugin_id: String, event_bus: Arc<EventBus>) -> Self {
        Self {
            plugin_id,
            event_bus,
        }
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn on<E: Event>(
        &self,
        priority: EventPriority,
        handler: impl Fn(&E) -> E + Send + Sync + 'static,
    ) {
        self.event_bus.register(priority, handler);
    }
}

pub struct PluginManager {
    plugins: DashMap<String, Box<dyn Plugin>>,
    contexts: DashMap<String, Context>,
    event_bus: Arc<EventBus>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
            contexts: DashMap::new(),
            event_bus: Arc::new(EventBus::new()),
        }
    }

    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    pub fn register(&self, plugin: Box<dyn Plugin>) {
        let meta = plugin.metadata();
        let id: String = meta.id.into();

        let ctx = Context::new(id.clone(), self.event_bus.clone());

        plugin.on_load(&ctx);

        self.plugins.insert(id.clone(), plugin);
        self.contexts.insert(id, ctx);
    }

    pub fn is_loaded(&self, id: &str) -> bool {
        self.plugins.contains_key(id)
    }

    pub fn loaded_plugins(&self) -> Vec<String> {
        self.plugins.iter().map(|e| e.key().clone()).collect()
    }

    pub fn load_plugins_from_dir(&self, dir: &str) -> Vec<String> {
        use crate::loader::NativePluginLoader;

        let path = Path::new(dir);
        if !path.exists() {
            return Vec::new();
        }

        let loader = NativePluginLoader;
        let mut loaded = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() && loader.can_load(&entry_path) {
                    match loader.load(&entry_path) {
                        Ok((plugin, meta, _)) => {
                            let id: String = meta.id.into();
                            self.register(plugin);
                            loaded.push(id);
                        }
                        Err(e) => {
                            eprintln!("Failed to load plugin from {:?}: {e}", entry_path);
                        }
                    }
                }
            }
        }

        loaded
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
