use neorusty_core::resource::ResourceLocation;
use std::collections::HashMap;

pub const BUCKET_VOLUME: u32 = 1000;

#[derive(Debug, Clone)]
pub struct FluidStack {
    fluid: ResourceLocation,
    amount: u32,
    components: HashMap<String, String>,
}

impl FluidStack {
    pub fn new(fluid: ResourceLocation, amount: u32) -> Self {
        Self {
            fluid,
            amount,
            components: HashMap::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            fluid: ResourceLocation::new("minecraft", "empty"),
            amount: 0,
            components: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.amount == 0
    }

    pub fn fluid(&self) -> &ResourceLocation {
        &self.fluid
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn set_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    pub fn with_amount(&self, amount: u32) -> Self {
        Self {
            amount,
            ..self.clone()
        }
    }

    pub fn split(&mut self, amount: u32) -> Self {
        let taken = self.amount.min(amount);
        self.amount -= taken;
        self.with_amount(taken)
    }

    pub fn matches(&self, other: &FluidStack) -> bool {
        self.fluid == other.fluid && self.components == other.components
    }
}

#[derive(Debug, Clone)]
pub struct FluidType {
    id: ResourceLocation,
    density: i32,
    temperature: i32,
    viscosity: i32,
    light_level: u32,
    motion_scale: f64,
    can_push_entity: bool,
    can_swim: bool,
    can_drown: bool,
    can_extinguish: bool,
}

impl FluidType {
    pub fn builder(id: ResourceLocation) -> FluidTypeBuilder {
        FluidTypeBuilder::new(id)
    }

    pub fn id(&self) -> &ResourceLocation {
        &self.id
    }

    pub fn density(&self) -> i32 {
        self.density
    }

    pub fn temperature(&self) -> i32 {
        self.temperature
    }

    pub fn viscosity(&self) -> i32 {
        self.viscosity
    }

    pub fn light_level(&self) -> u32 {
        self.light_level
    }
}

pub struct FluidTypeBuilder {
    id: ResourceLocation,
    density: i32,
    temperature: i32,
    viscosity: i32,
    light_level: u32,
    motion_scale: f64,
    can_push_entity: bool,
    can_swim: bool,
    can_drown: bool,
    can_extinguish: bool,
}

impl FluidTypeBuilder {
    pub fn new(id: ResourceLocation) -> Self {
        Self {
            id,
            density: 1000,
            temperature: 300,
            viscosity: 1000,
            light_level: 0,
            motion_scale: 0.014,
            can_push_entity: true,
            can_swim: true,
            can_drown: true,
            can_extinguish: false,
        }
    }

    pub fn density(mut self, v: i32) -> Self {
        self.density = v;
        self
    }

    pub fn temperature(mut self, v: i32) -> Self {
        self.temperature = v;
        self
    }

    pub fn viscosity(mut self, v: i32) -> Self {
        self.viscosity = v;
        self
    }

    pub fn light_level(mut self, v: u32) -> Self {
        self.light_level = v;
        self
    }

    pub fn can_extinguish(mut self, v: bool) -> Self {
        self.can_extinguish = v;
        self
    }

    pub fn build(self) -> FluidType {
        FluidType {
            id: self.id,
            density: self.density,
            temperature: self.temperature,
            viscosity: self.viscosity,
            light_level: self.light_level,
            motion_scale: self.motion_scale,
            can_push_entity: self.can_push_entity,
            can_swim: self.can_swim,
            can_drown: self.can_drown,
            can_extinguish: self.can_extinguish,
        }
    }
}
