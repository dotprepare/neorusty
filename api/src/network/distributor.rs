use super::payload::CustomPacketPayload;
use std::sync::Arc;

pub struct PacketDistributor {
    write_fn: Arc<dyn Fn(&[u8]) -> Result<(), String> + Send + Sync>,
}

impl PacketDistributor {
    pub fn new(
        write_fn: impl Fn(&[u8]) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            write_fn: Arc::new(write_fn),
        }
    }

    pub fn send<T: CustomPacketPayload>(&self, payload: &T) -> Result<(), String> {
        let mut buf = Vec::new();
        let id = payload.id().to_string();
        let id_bytes = id.as_bytes();
        let len = id_bytes.len() as u16;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(id_bytes);
        (self.write_fn)(&buf)
    }

    pub fn send_to_player<T: CustomPacketPayload>(
        &self,
        _player_id: &str,
        payload: &T,
    ) -> Result<(), String> {
        self.send(payload)
    }

    pub fn broadcast<T: CustomPacketPayload>(&self, payload: &T) -> Result<(), String> {
        self.send(payload)
    }
}
