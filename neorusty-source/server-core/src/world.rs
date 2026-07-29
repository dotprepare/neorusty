use neorusty_core::resource::ResourceLocation;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Easy
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub id: ResourceLocation,
    pub name: String,
    pub seed: u64,
    pub difficulty: Difficulty,
    pub time: u64,
    pub raining: bool,
    pub thundering: bool,
}

impl World {
    pub fn new(id: ResourceLocation, name: String, seed: u64) -> Self {
        Self {
            id,
            name,
            seed,
            difficulty: Difficulty::default(),
            time: 0,
            raining: false,
            thundering: false,
        }
    }

    pub fn tick(&mut self) {
        self.time = self.time.wrapping_add(1);
    }
}

pub struct WorldManager {
    worlds: HashMap<ResourceLocation, World>,
    primary: Option<ResourceLocation>,
}

impl WorldManager {
    pub fn new() -> Self {
        Self {
            worlds: HashMap::new(),
            primary: None,
        }
    }

    pub fn add_world(&mut self, world: World) {
        let id = world.id.clone();
        if self.primary.is_none() {
            self.primary = Some(id.clone());
        }
        self.worlds.insert(id, world);
    }

    pub fn get(&self, id: &ResourceLocation) -> Option<&World> {
        self.worlds.get(id)
    }

    pub fn get_mut(&mut self, id: &ResourceLocation) -> Option<&mut World> {
        self.worlds.get_mut(id)
    }

    pub fn primary(&self) -> Option<&World> {
        self.primary.as_ref().and_then(|id| self.worlds.get(id))
    }

    pub fn primary_mut(&mut self) -> Option<&mut World> {
        let id = self.primary.clone()?;
        self.worlds.get_mut(&id)
    }

    pub fn all(&self) -> impl Iterator<Item = &World> {
        self.worlds.values()
    }

    pub fn remove(&mut self, id: &ResourceLocation) -> bool {
        self.worlds.remove(id).is_some()
    }

    pub fn tick_all(&mut self) {
        for world in self.worlds.values_mut() {
            world.tick();
        }
    }
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new()
    }
}
