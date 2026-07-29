use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub registry_type: String,
    pub id: String,
    pub data: Vec<u8>,
}

impl RegistryEntry {
    pub fn new(registry_type: &str, id: &str, data: Vec<u8>) -> Self {
        Self {
            registry_type: registry_type.to_string(),
            id: id.to_string(),
            data,
        }
    }
}

struct RegistryStore {
    entries: Vec<RegistryEntry>,
}

impl RegistryStore {
    const fn new() -> Self {
        Self { entries: Vec::new() }
    }

    fn push(&mut self, entry: RegistryEntry) {
        self.entries.push(entry);
    }

    fn get_all(&self) -> Vec<RegistryEntry> {
        self.entries.clone()
    }

    fn count(&self) -> usize {
        self.entries.len()
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

static REGISTRY: Mutex<RegistryStore> = Mutex::new(RegistryStore::new());

pub fn push_entry(entry: RegistryEntry) {
    if let Ok(mut store) = REGISTRY.lock() {
        store.push(entry);
    }
}

pub fn get_entries() -> Vec<RegistryEntry> {
    REGISTRY
        .lock()
        .map(|store| store.get_all())
        .unwrap_or_default()
}

pub fn total_count() -> usize {
    REGISTRY
        .lock()
        .map(|store| store.count())
        .unwrap_or(0)
}

pub fn clear() {
    if let Ok(mut store) = REGISTRY.lock() {
        store.clear();
    }
}
