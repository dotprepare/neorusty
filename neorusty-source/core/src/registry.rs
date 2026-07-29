use crate::resource::{ResourceKey, ResourceLocation};
use dashmap::DashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum RegistryHandle<T> {
    Reference(ResourceKey<T>),
    Direct(Arc<T>),
}

impl<T> RegistryHandle<T> {
    pub fn get<Reg: Registry<T>>(&self, registry: &Reg) -> Option<Arc<T>>
    where
        T: 'static,
    {
        match self {
            RegistryHandle::Direct(t) => Some(t.clone()),
            RegistryHandle::Reference(key) => registry.get(key.location()),
        }
    }

    pub fn key(&self) -> Option<&ResourceKey<T>> {
        match self {
            RegistryHandle::Reference(key) => Some(key),
            RegistryHandle::Direct(_) => None,
        }
    }
}

pub trait Registry<T>: Send + Sync {
    fn register(&self, key: &ResourceLocation, value: T) -> Option<T>;
    fn get(&self, key: &ResourceLocation) -> Option<Arc<T>>;
    fn contains(&self, key: &ResourceLocation) -> bool;
    fn iter(&self) -> Vec<(ResourceLocation, Arc<T>)>;
    fn keys(&self) -> Vec<ResourceLocation>;
    fn clear(&self);
}

pub struct MappedRegistry<T> {
    inner: DashMap<ResourceLocation, Arc<T>>,
}

impl<T> MappedRegistry<T> {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }
}

impl<T: Send + Sync + 'static> Registry<T> for MappedRegistry<T> {
    fn register(&self, key: &ResourceLocation, value: T) -> Option<T> {
        self.inner.insert(key.clone(), Arc::new(value));
        None
    }

    fn get(&self, key: &ResourceLocation) -> Option<Arc<T>> {
        self.inner.get(key).map(|v| v.clone())
    }

    fn contains(&self, key: &ResourceLocation) -> bool {
        self.inner.contains_key(key)
    }

    fn iter(&self) -> Vec<(ResourceLocation, Arc<T>)> {
        self.inner
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    fn keys(&self) -> Vec<ResourceLocation> {
        self.inner.iter().map(|e| e.key().clone()).collect()
    }

    fn clear(&self) {
        self.inner.clear();
    }
}
