use neorusty_core::resource::ResourceLocation;
use std::marker::PhantomData;

pub struct Capability<T: ?Sized, C = ()> {
    name: ResourceLocation,
    _marker: PhantomData<(Box<T>, C)>,
}

impl<T: ?Sized, C> Capability<T, C> {
    pub fn new(name: ResourceLocation) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &ResourceLocation {
        &self.name
    }
}

impl<T: ?Sized, C> Clone for Capability<T, C> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            _marker: PhantomData,
        }
    }
}

pub type BlockCapability<T, C = ()> = Capability<T, C>;
pub type EntityCapability<T, C = ()> = Capability<T, C>;
pub type ItemCapability<T, C = ()> = Capability<T, C>;
