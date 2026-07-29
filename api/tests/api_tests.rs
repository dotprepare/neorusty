use neorusty_api::network::codec::{StreamCodec, VarIntCodec, StringCodec, UnitCodec};
use neorusty_api::network::payload::{CustomPacketPayload, PayloadContext, PayloadRegistrar, PacketFlow};
use std::io::{Read, Write};
use neorusty_api::registry::{DeferredHolder, DeferredRegister};
use neorusty_api::energy::{BasicEnergyStorage, EnergyStorage};
use neorusty_api::fluid::{FluidStack, FluidType, BUCKET_VOLUME};
use neorusty_api::item::{BasicItemHandler, ItemHandler, ItemStack};
use neorusty_core::registry::{MappedRegistry, Registry};
use neorusty_core::resource::ResourceLocation;

#[test]
fn test_deferred_register() {
    let reg: MappedRegistry<String> = MappedRegistry::new();
    let dr = DeferredRegister::<String>::new("testmod", ResourceLocation::parse("minecraft:test").unwrap());

    let _holder: DeferredHolder<String> = dr.register("hello", || "world".to_string());
    let _holder2 = dr.register("foo", || "bar".to_string());

    dr.register_all(&reg);

    assert_eq!(
        reg.get(&ResourceLocation::parse("testmod:hello").unwrap()),
        Some(std::sync::Arc::new("world".to_string()))
    );
    assert_eq!(
        reg.get(&ResourceLocation::parse("testmod:foo").unwrap()),
        Some(std::sync::Arc::new("bar".to_string()))
    );
}

#[test]
fn test_energy_storage() {
    let mut storage = BasicEnergyStorage::new(1000, 100, 100);

    assert_eq!(storage.energy_stored(), 0);
    assert_eq!(storage.receive_energy(50, false), 50);
    assert_eq!(storage.energy_stored(), 50);

    assert_eq!(storage.extract_energy(30, false), 30);
    assert_eq!(storage.energy_stored(), 20);

    let received = storage.receive_energy(200, false);
    assert_eq!(received, 100);
    assert_eq!(storage.energy_stored(), 120);
}

#[test]
fn test_energy_simulate() {
    let mut storage = BasicEnergyStorage::new(1000, 100, 100);

    assert_eq!(storage.receive_energy(50, true), 50);
    assert_eq!(storage.energy_stored(), 0);

    assert_eq!(storage.extract_energy(50, true), 0);
    assert_eq!(storage.energy_stored(), 0);
}

#[test]
fn test_fluid_stack() {
    let water = ResourceLocation::new("minecraft", "water");
    let stack = FluidStack::new(water.clone(), BUCKET_VOLUME);

    assert!(!stack.is_empty());
    assert_eq!(*stack.fluid(), water);
    assert_eq!(stack.amount(), BUCKET_VOLUME);

    let empty = FluidStack::empty();
    assert!(empty.is_empty());
}

#[test]
fn test_item_handler() {
    let mut handler = BasicItemHandler::new(9, 64);
    assert_eq!(handler.slots(), 9);

    let apple = ResourceLocation::new("minecraft", "apple");
    let stack = ItemStack::new(apple.clone(), 32);

    let remainder = handler.insert_item(0, stack);
    assert!(remainder.is_empty());

    let retrieved = handler.stack_in_slot(0);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().count(), 32);

    let extracted = handler.extract_item(0, 10);
    assert_eq!(extracted.count(), 10);

    let retrieved2 = handler.stack_in_slot(0);
    assert_eq!(retrieved2.unwrap().count(), 22);
}

#[test]
fn test_fluid_type_builder() {
    let id = ResourceLocation::new("minecraft", "lava");
    let lava = FluidType::builder(id.clone())
        .density(3000)
        .temperature(1300)
        .viscosity(6000)
        .light_level(15)
        .build();

    assert_eq!(*lava.id(), id);
    assert_eq!(lava.density(), 3000);
    assert_eq!(lava.temperature(), 1300);
    assert_eq!(lava.light_level(), 15);
}

#[test]
fn test_varint_codec_roundtrip() {
    let codec = VarIntCodec;
    let mut buf = Vec::new();
    <VarIntCodec as StreamCodec<i32>>::encode(&codec, &mut buf, &0).unwrap();
    assert_eq!(<VarIntCodec as StreamCodec<i32>>::decode(&codec, &mut &buf[..]).unwrap(), 0);

    buf.clear();
    <VarIntCodec as StreamCodec<i32>>::encode(&codec, &mut buf, &127).unwrap();
    assert_eq!(<VarIntCodec as StreamCodec<i32>>::decode(&codec, &mut &buf[..]).unwrap(), 127);

    buf.clear();
    <VarIntCodec as StreamCodec<i32>>::encode(&codec, &mut buf, &255).unwrap();
    assert_eq!(<VarIntCodec as StreamCodec<i32>>::decode(&codec, &mut &buf[..]).unwrap(), 255);

    buf.clear();
    <VarIntCodec as StreamCodec<i32>>::encode(&codec, &mut buf, &i32::MAX).unwrap();
    assert_eq!(<VarIntCodec as StreamCodec<i32>>::decode(&codec, &mut &buf[..]).unwrap(), i32::MAX);
}

#[test]
fn test_string_codec_roundtrip() {
    let codec = StringCodec;
    let mut buf = Vec::new();
    codec.encode(&mut buf, &"hello".to_string()).unwrap();
    let decoded: String = codec.decode(&mut &buf[..]).unwrap();
    assert_eq!(decoded, "hello");
}

#[test]
fn test_unit_codec() {
    let codec = UnitCodec;
    let mut buf = Vec::new();
    codec.encode(&mut buf, &()).unwrap();
    assert!(codec.decode(&mut &buf[..]).is_ok());
}

#[derive(Debug, Clone)]
struct TestPayload {
    id: ResourceLocation,
    value: i32,
}

impl CustomPacketPayload for TestPayload {
    fn id(&self) -> &ResourceLocation {
        &self.id
    }
}

impl StreamCodec<TestPayload> for VarIntCodec {
    fn encode(&self, buf: &mut dyn Write, value: &TestPayload) -> Result<(), String> {
        VarIntCodec.encode(buf, &value.value)
    }

    fn decode(&self, buf: &mut dyn Read) -> Result<TestPayload, String> {
        let value = VarIntCodec.decode(buf)?;
        Ok(TestPayload {
            id: ResourceLocation::new("testmod", "test"),
            value,
        })
    }
}

#[test]
fn test_payload_registration_and_dispatch() {
    let mut registrar = PayloadRegistrar::new();
    let id = ResourceLocation::new("testmod", "test");

    registrar.register(
        id.clone(),
        PacketFlow::Bidirectional,
        VarIntCodec,
        |payload: TestPayload, _ctx: &PayloadContext| {
            assert_eq!(payload.value, 42);
            Ok(())
        },
    );

    assert!(registrar.contains(&id));

    let ctx = PayloadContext::new("test_conn".to_string());
    let mut buf = Vec::new();
    let payload = TestPayload {
        id: id.clone(),
        value: 42,
    };
    VarIntCodec.encode(&mut buf, &payload).unwrap();
    registrar
        .dispatch(&id, &mut &buf[..], &ctx)
        .unwrap();
}

#[test]
fn test_payload_unregistered_handler() {
    let registrar = PayloadRegistrar::new();
    let id = ResourceLocation::new("testmod", "unknown");
    let ctx = PayloadContext::new("test".to_string());

    let result = registrar.dispatch(&id, &mut &[][..], &ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No handler registered"));
}

#[test]
fn test_payload_flow_enum() {
    assert_eq!(PacketFlow::Clientbound as u8, 0);
    assert_eq!(PacketFlow::Serverbound as u8, 1);
    assert_eq!(PacketFlow::Bidirectional as u8, 2);
}
