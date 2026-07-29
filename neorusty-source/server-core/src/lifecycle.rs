use neorusty_core::event::Event;

#[derive(Debug, Clone)]
pub struct ServerStartingEvent;

impl Event for ServerStartingEvent {}

#[derive(Debug, Clone)]
pub struct ServerStartedEvent;

impl Event for ServerStartedEvent {}

#[derive(Debug, Clone)]
pub struct ServerStoppingEvent;

impl Event for ServerStoppingEvent {}

#[derive(Debug, Clone)]
pub struct ServerTickEvent {
    pub tick: u64,
}

impl Event for ServerTickEvent {}
