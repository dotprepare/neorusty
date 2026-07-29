use super::codec::StreamCodec;
use neorusty_core::resource::ResourceLocation;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;

pub trait CustomPacketPayload: Send + Sync + 'static {
    fn id(&self) -> &ResourceLocation;
}

pub struct PayloadType<T: CustomPacketPayload> {
    pub id: ResourceLocation,
    pub codec: Arc<dyn StreamCodec<T>>,
}

pub struct PayloadRegistration<T: CustomPacketPayload> {
    pub payload_type: PayloadType<T>,
    pub direction: PacketFlow,
    pub handler: Arc<dyn Fn(T, &PayloadContext) -> Result<(), String> + Send + Sync>,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
    Bidirectional,
}

#[derive(Debug, Clone)]
pub struct PayloadContext {
    pub player_id: Option<String>,
    pub connection_id: String,
}

impl PayloadContext {
    pub fn new(connection_id: String) -> Self {
        Self {
            player_id: None,
            connection_id,
        }
    }

    pub fn with_player(mut self, player_id: String) -> Self {
        self.player_id = Some(player_id);
        self
    }
}

pub struct PayloadRegistrar {
    registrations: HashMap<ResourceLocation, Box<dyn AnyRegistration + Send + Sync>>,
}

#[allow(dead_code)]
trait AnyRegistration: Send + Sync {
    fn id(&self) -> &ResourceLocation;
    fn direction(&self) -> PacketFlow;
    fn handle_any(&self, buf: &mut dyn Read, ctx: &PayloadContext) -> Result<(), String>;
}

impl<T: CustomPacketPayload + 'static> AnyRegistration for PayloadRegistration<T> {
    fn id(&self) -> &ResourceLocation {
        &self.payload_type.id
    }

    fn direction(&self) -> PacketFlow {
        self.direction
    }

    fn handle_any(&self, buf: &mut dyn Read, ctx: &PayloadContext) -> Result<(), String> {
        let value = self.payload_type.codec.decode(buf)?;
        (self.handler)(value, ctx)
    }
}

impl PayloadRegistrar {
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
        }
    }

    pub fn register<T: CustomPacketPayload + 'static>(
        &mut self,
        id: ResourceLocation,
        direction: PacketFlow,
        codec: impl StreamCodec<T>,
        handler: impl Fn(T, &PayloadContext) -> Result<(), String> + Send + Sync + 'static,
    ) -> &mut Self {
        self.registrations.insert(
            id.clone(),
            Box::new(PayloadRegistration {
                payload_type: PayloadType {
                    id,
                    codec: Arc::new(codec),
                },
                direction,
                handler: Arc::new(handler),
                optional: false,
            }),
        );
        self
    }

    pub fn dispatch(
        &self,
        id: &ResourceLocation,
        buf: &mut dyn Read,
        ctx: &PayloadContext,
    ) -> Result<(), String> {
        match self.registrations.get(id) {
            Some(reg) => reg.handle_any(buf, ctx),
            None => Err(format!("No handler registered for payload {id}")),
        }
    }

    pub fn contains(&self, id: &ResourceLocation) -> bool {
        self.registrations.contains_key(id)
    }

    pub fn registered_ids(&self) -> Vec<ResourceLocation> {
        self.registrations.keys().cloned().collect()
    }
}

impl Default for PayloadRegistrar {
    fn default() -> Self {
        Self::new()
    }
}
