use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct BridgeEvent {
    pub event_type: String,
    pub data: Vec<u8>,
    pub direction: String,
}

struct EventStore {
    events: Vec<BridgeEvent>,
}

impl EventStore {
    const fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn push(&mut self, event: BridgeEvent) {
        self.events.push(event);
    }

    fn drain(&mut self) -> Vec<BridgeEvent> {
        std::mem::take(&mut self.events)
    }

    fn count(&self) -> usize {
        self.events.len()
    }

    fn clear(&mut self) {
        self.events.clear();
    }
}

static EVENTS: Mutex<EventStore> = Mutex::new(EventStore::new());

pub fn push_event(event_type: &str, data: Vec<u8>, direction: &str) {
    if let Ok(mut store) = EVENTS.lock() {
        store.push(BridgeEvent {
            event_type: event_type.to_string(),
            data,
            direction: direction.to_string(),
        });
    }
}

pub fn drain_events() -> Vec<BridgeEvent> {
    EVENTS
        .lock()
        .map(|mut store| store.drain())
        .unwrap_or_default()
}

pub fn event_count() -> usize {
    EVENTS
        .lock()
        .map(|store| store.count())
        .unwrap_or(0)
}

pub fn clear() {
    if let Ok(mut store) = EVENTS.lock() {
        store.clear();
    }
}
