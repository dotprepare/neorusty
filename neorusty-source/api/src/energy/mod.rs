use std::sync::atomic::{AtomicU32, Ordering};

pub trait EnergyStorage: Send + Sync {
    fn receive_energy(&mut self, to_receive: u32, simulate: bool) -> u32;
    fn extract_energy(&mut self, to_extract: u32, simulate: bool) -> u32;
    fn energy_stored(&self) -> u32;
    fn max_energy_stored(&self) -> u32;
    fn can_extract(&self) -> bool;
    fn can_receive(&self) -> bool;
}

pub struct BasicEnergyStorage {
    energy: AtomicU32,
    capacity: u32,
    max_receive: u32,
    max_extract: u32,
}

impl BasicEnergyStorage {
    pub fn new(capacity: u32, max_receive: u32, max_extract: u32) -> Self {
        Self {
            energy: AtomicU32::new(0),
            capacity,
            max_receive,
            max_extract,
        }
    }

    pub fn with_energy(mut self, energy: u32) -> Self {
        self.energy = AtomicU32::new(energy);
        self
    }
}

impl EnergyStorage for BasicEnergyStorage {
    fn receive_energy(&mut self, to_receive: u32, simulate: bool) -> u32 {
        let current = self.energy.load(Ordering::Acquire);
        let space = self.capacity.saturating_sub(current);
        let received = to_receive.min(self.max_receive).min(space);

        if !simulate && received > 0 {
            self.energy.store(current + received, Ordering::Release);
        }

        received
    }

    fn extract_energy(&mut self, to_extract: u32, simulate: bool) -> u32 {
        let current = self.energy.load(Ordering::Acquire);
        let extracted = to_extract.min(self.max_extract).min(current);

        if !simulate && extracted > 0 {
            self.energy.store(current - extracted, Ordering::Release);
        }

        extracted
    }

    fn energy_stored(&self) -> u32 {
        self.energy.load(Ordering::Acquire)
    }

    fn max_energy_stored(&self) -> u32 {
        self.capacity
    }

    fn can_extract(&self) -> bool {
        self.max_extract > 0
    }

    fn can_receive(&self) -> bool {
        self.max_receive > 0
    }
}
