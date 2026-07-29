use neorusty_core::registry::MappedRegistry;
use neorusty_core::resource::ResourceLocation;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

type RegistryEntry = Arc<dyn std::any::Any + Send + Sync>;

pub struct RegistryManager {
    registries: RwLock<HashMap<ResourceLocation, RegistryEntry>>,
    frozen: RwLock<bool>,
}

impl RegistryManager {
    pub fn new() -> Self {
        Self {
            registries: RwLock::new(HashMap::new()),
            frozen: RwLock::new(false),
        }
    }

    pub fn register_registry<T: Send + Sync + 'static>(
        &self,
        key: ResourceLocation,
        registry: MappedRegistry<T>,
    ) {
        if *self.frozen.read() {
            panic!("Cannot register registry after freeze: {key}");
        }
        self.registries
            .write()
            .insert(key, Arc::new(registry));
    }

    pub fn get<T: Send + Sync + 'static>(&self, key: &ResourceLocation) -> Option<Arc<MappedRegistry<T>>> {
        self.registries
            .read()
            .get(key)
            .and_then(|entry| entry.clone().downcast::<MappedRegistry<T>>().ok())
    }

    pub fn freeze(&self) {
        *self.frozen.write() = true;
    }

    pub fn is_frozen(&self) -> bool {
        *self.frozen.read()
    }

    pub fn all_keys(&self) -> Vec<ResourceLocation> {
        self.registries.read().keys().cloned().collect()
    }
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self::new()
    }
}
