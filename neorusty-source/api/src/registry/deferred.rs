use neorusty_core::registry::Registry;
use neorusty_core::resource::ResourceLocation;
use parking_lot::RwLock;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct DeferredRegister<T> {
    mod_id: String,
    registry_key: ResourceLocation,
    entries: RwLock<Vec<DeferredEntry<T>>>,
}

struct DeferredEntry<T> {
    name: ResourceLocation,
    supplier: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Send + Sync + 'static> DeferredRegister<T> {
    pub fn new(mod_id: impl Into<String>, registry_key: ResourceLocation) -> Self {
        Self {
            mod_id: mod_id.into(),
            registry_key,
            entries: RwLock::new(Vec::new()),
        }
    }

    pub fn register(
        &self,
        name: impl Into<ResourceLocation>,
        supplier: impl Fn() -> T + Send + Sync + 'static,
    ) -> DeferredHolder<T> {
        let raw: ResourceLocation = name.into();
        let full_name = if raw.namespace() == "minecraft" || raw.namespace().is_empty() {
            ResourceLocation::new(&self.mod_id, raw.path())
        } else {
            raw
        };

        self.entries.write().push(DeferredEntry {
            name: full_name.clone(),
            supplier: Box::new(supplier),
        });

        DeferredHolder {
            key: full_name,
            _marker: PhantomData,
        }
    }

    pub fn register_all(&self, registry: &dyn Registry<T>) {
        let entries = self.entries.read();
        for entry in entries.iter() {
            let value = (entry.supplier)();
            registry.register(&entry.name, value);
        }
    }

    pub fn mod_id(&self) -> &str {
        &self.mod_id
    }

    pub fn registry_key(&self) -> &ResourceLocation {
        &self.registry_key
    }
}

pub struct DeferredHolder<T> {
    key: ResourceLocation,
    _marker: PhantomData<T>,
}

impl<T> DeferredHolder<T> {
    pub fn key(&self) -> &ResourceLocation {
        &self.key
    }

    pub fn get(&self, registry: &dyn Registry<T>) -> Option<Arc<T>>
    where
        T: 'static,
    {
        registry.get(&self.key)
    }
}

impl<T> Clone for DeferredHolder<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: PhantomData,
        }
    }
}
