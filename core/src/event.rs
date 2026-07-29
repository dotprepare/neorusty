use crate::tri_state::TriState;
use dashmap::DashMap;
use std::any::Any;
use std::any::TypeId;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Lowest = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Highest = 4,
    Monitor = 5,
}

pub trait Event: Any + Send + Sync + 'static {
    fn is_cancellable(&self) -> bool {
        false
    }

    fn is_cancelled(&self) -> bool {
        false
    }

    fn set_cancelled(&mut self, _cancelled: bool) {}
}

pub trait HasResult: Event {
    fn result(&self) -> TriState;
    fn set_result(&mut self, result: TriState);
}

pub trait Cancellable: Event {
    fn cancel(&mut self);
    fn uncancel(&mut self);
}

/// Sequential handler: transforms event by value.
type SeqHandler<E> = Box<dyn Fn(&E) -> E + Send + Sync>;

/// Parallel handler: receives shared event for in-place mutation.
type ParHandler<E> = Box<dyn Fn(&parking_lot::RwLock<E>) + Send + Sync>;

enum HandlerSlot<E: Event> {
    Sequential { priority: EventPriority, handler: SeqHandler<E> },
    Parallel { priority: EventPriority, handler: ParHandler<E> },
}

pub struct EventBus {
    handlers: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
        }
    }

    pub fn register<E: Event>(
        &self,
        priority: EventPriority,
        handler: impl Fn(&E) -> E + Send + Sync + 'static,
    ) {
        let tid = TypeId::of::<E>();
        let slot = HandlerSlot::Sequential {
            priority,
            handler: Box::new(handler),
        };
        self.push_handler::<E>(tid, slot);
    }

    pub fn register_parallel<E: Event>(
        &self,
        priority: EventPriority,
        handler: impl Fn(&parking_lot::RwLock<E>) + Send + Sync + 'static,
    ) {
        let tid = TypeId::of::<E>();
        let slot = HandlerSlot::Parallel {
            priority,
            handler: Box::new(handler),
        };
        self.push_handler::<E>(tid, slot);
    }

    fn push_handler<E: Event>(&self, tid: TypeId, slot: HandlerSlot<E>) {
        let mut entry = self.handlers.entry(tid).or_insert_with(|| {
            Box::new(Vec::<HandlerSlot<E>>::new()) as Box<dyn Any + Send + Sync>
        });
        let slots = entry.downcast_mut::<Vec<HandlerSlot<E>>>().expect("wrong type");
        slots.push(slot);
        slots.sort_by_key(|s| match s {
            HandlerSlot::Sequential { priority, .. } => *priority,
            HandlerSlot::Parallel { priority, .. } => *priority,
        });
    }

    /// Sequential dispatch: each handler receives the output of the previous.
    pub fn post<E: Event>(&self, event: E) -> E {
        let tid = TypeId::of::<E>();
        let guard = self.handlers.get(&tid);
        let Some(entry) = guard else {
            return event;
        };
        let slots = entry.downcast_ref::<Vec<HandlerSlot<E>>>().expect("wrong type");

        let mut current = event;
        for slot in slots {
            match slot {
                HandlerSlot::Sequential { handler, .. } => {
                    current = handler(&current);
                }
                HandlerSlot::Parallel { handler, .. } => {
                    let shared = parking_lot::RwLock::new(current);
                    handler(&shared);
                    current = shared.into_inner();
                }
            }
        }
        current
    }

    /// Parallel-aware dispatch.
    /// Groups handlers by priority. Within each group, parallel handlers
    /// share the event via RwLock and run concurrently via rayon.
    pub fn post_parallel<E: Event + Clone + Sync>(&self, event: E) -> E {
        let tid = TypeId::of::<E>();
        let guard = self.handlers.get(&tid);
        let Some(entry) = guard else {
            return event;
        };
        let slots = entry.downcast_ref::<Vec<HandlerSlot<E>>>().expect("wrong type");

        let mut current = event;
        let mut i = 0;

        while i < slots.len() {
            let current_priority = match &slots[i] {
                HandlerSlot::Sequential { priority, .. } => *priority,
                HandlerSlot::Parallel { priority, .. } => *priority,
            };

            let group_start = i;
            let mut group_end = group_start;
            let mut has_parallel = false;

            while group_end < slots.len() {
                let p = match &slots[group_end] {
                    HandlerSlot::Sequential { priority, .. } => *priority,
                    HandlerSlot::Parallel { priority, .. } => *priority,
                };
                if p != current_priority {
                    break;
                }
                if matches!(&slots[group_end], HandlerSlot::Parallel { .. }) {
                    has_parallel = true;
                }
                group_end += 1;
            }

            if has_parallel {
                let shared = Arc::new(parking_lot::RwLock::new(current));
                let mut parallel_handlers: Vec<&ParHandler<E>> = Vec::new();

                for j in group_start..group_end {
                    match &slots[j] {
                        HandlerSlot::Sequential { handler, .. } => {
                            let mut guard = shared.write();
                            let cloned = guard.clone();
                            *guard = handler(&cloned);
                        }
                        HandlerSlot::Parallel { handler, .. } => {
                            parallel_handlers.push(handler);
                        }
                    }
                }

                rayon::scope(|s| {
                    for handler in parallel_handlers {
                        let shared = shared.clone();
                        s.spawn(move |_| {
                            handler(&shared);
                        });
                    }
                });

                current = Arc::into_inner(shared)
                    .expect("RwLock still referenced")
                    .into_inner();
            } else {
                for j in group_start..group_end {
                    match &slots[j] {
                        HandlerSlot::Sequential { handler, .. } => {
                            current = handler(&current);
                        }
                        HandlerSlot::Parallel { handler, .. } => {
                            let shared = parking_lot::RwLock::new(current);
                            handler(&shared);
                            current = shared.into_inner();
                        }
                    }
                }
            }

            i = group_end;
        }

        current
    }

    pub fn clear<E: Event>(&self) {
        let tid = TypeId::of::<E>();
        self.handlers.remove(&tid);
    }

    pub fn registered_count<E: Event>(&self) -> usize {
        let tid = TypeId::of::<E>();
        self.handlers
            .get(&tid)
            .map(|entry| {
                let slots = entry.downcast_ref::<Vec<HandlerSlot<E>>>().expect("wrong type");
                slots.len()
            })
            .unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestEvent {
        value: i32,
    }

    impl Event for TestEvent {}

    #[derive(Debug, Clone, PartialEq)]
    struct CounterEvent {
        seen: Vec<i32>,
    }

    impl Event for CounterEvent {}

    #[test]
    fn test_post_no_handlers() {
        let bus = EventBus::new();
        let event = TestEvent { value: 42 };
        let result = bus.post(event);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_single_handler() {
        let bus = EventBus::new();
        bus.register(EventPriority::Normal, |e: &TestEvent| TestEvent {
            value: e.value + 1,
        });

        let event = TestEvent { value: 41 };
        let result = bus.post(event);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_priority_order() {
        let bus = EventBus::new();
        bus.register(EventPriority::Low, |e: &TestEvent| TestEvent {
            value: e.value + 10,
        });
        bus.register(EventPriority::High, |e: &TestEvent| TestEvent {
            value: e.value * 2,
        });

        let event = TestEvent { value: 5 };
        let result = bus.post(event);
        assert_eq!(result.value, 30);
    }

    #[test]
    fn test_clear() {
        let bus = EventBus::new();
        bus.register(EventPriority::Normal, |e: &TestEvent| TestEvent {
            value: e.value + 1,
        });

        assert_eq!(bus.registered_count::<TestEvent>(), 1);
        bus.clear::<TestEvent>();
        assert_eq!(bus.registered_count::<TestEvent>(), 0);

        let event = TestEvent { value: 42 };
        let result = bus.post(event);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_parallel_dispatch_mutation() {
        let bus = EventBus::new();

        bus.register_parallel::<CounterEvent>(EventPriority::Normal, |shared| {
            let mut e = shared.write();
            e.seen.push(1);
        });

        bus.register_parallel::<CounterEvent>(EventPriority::Normal, |shared| {
            let mut e = shared.write();
            e.seen.push(2);
        });

        bus.register_parallel::<CounterEvent>(EventPriority::Normal, |shared| {
            let mut e = shared.write();
            e.seen.push(3);
        });

        let event = CounterEvent { seen: vec![] };
        let result = bus.post_parallel(event);

        assert_eq!(result.seen.len(), 3);
        let mut sorted = result.seen.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3]);
    }

    #[test]
    fn test_parallel_mixed_with_sequential() {
        let bus = EventBus::new();

        // Sequential: doubles the value
        bus.register::<TestEvent>(EventPriority::Low, |e: &TestEvent| TestEvent {
            value: e.value * 2,
        });

        // Parallel: adds 1
        bus.register_parallel::<TestEvent>(EventPriority::Normal, |shared| {
            let mut e = shared.write();
            e.value += 1;
        });

        let event = TestEvent { value: 10 };
        let result = bus.post_parallel(event);

        // Low priority (sequential) runs first: 10 * 2 = 20
        // Then Normal (parallel) runs: 20 + 1 = 21
        assert_eq!(result.value, 21);
    }

    #[test]
    fn test_post_parallel_falls_back_to_sequential() {
        let bus = EventBus::new();
        bus.register(EventPriority::Normal, |e: &TestEvent| TestEvent {
            value: e.value * 3,
        });

        let event = TestEvent { value: 10 };
        let result = bus.post_parallel(event);
        assert_eq!(result.value, 30);
    }
}
