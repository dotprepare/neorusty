use neorusty_core::resource::ResourceLocation;
use std::sync::Mutex;

#[derive(Clone)]
pub struct ItemStack {
    item: ResourceLocation,
    count: u32,
    components: std::collections::HashMap<String, String>,
}

impl ItemStack {
    pub fn new(item: ResourceLocation, count: u32) -> Self {
        Self {
            item,
            count,
            components: std::collections::HashMap::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            item: ResourceLocation::new("minecraft", "air"),
            count: 0,
            components: std::collections::HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn item(&self) -> &ResourceLocation {
        &self.item
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    pub fn split(&mut self, amount: u32) -> Self {
        let taken = self.count.min(amount);
        self.count -= taken;
        Self {
            item: self.item.clone(),
            count: taken,
            components: self.components.clone(),
        }
    }
}

pub trait ItemHandler: Send + Sync {
    fn slots(&self) -> u32;
    fn stack_in_slot(&self, slot: u32) -> Option<ItemStack>;
    fn insert_item(&mut self, slot: u32, stack: ItemStack) -> ItemStack;
    fn extract_item(&mut self, slot: u32, amount: u32) -> ItemStack;
    fn slot_limit(&self, _slot: u32) -> u32;
}

pub struct BasicItemHandler {
    slots: Mutex<Vec<ItemStack>>,
    max_stack_size: u32,
}

impl BasicItemHandler {
    pub fn new(slot_count: u32, max_stack_size: u32) -> Self {
        Self {
            slots: Mutex::new((0..slot_count).map(|_| ItemStack::empty()).collect()),
            max_stack_size,
        }
    }
}

impl ItemHandler for BasicItemHandler {
    fn slots(&self) -> u32 {
        self.slots.lock().unwrap().len() as u32
    }

    fn stack_in_slot(&self, slot: u32) -> Option<ItemStack> {
        let slots = self.slots.lock().unwrap();
        let idx = slot as usize;
        if idx < slots.len() {
            let stack = &slots[idx];
            if stack.is_empty() {
                None
            } else {
                Some(ItemStack::new(stack.item().clone(), stack.count()))
            }
        } else {
            None
        }
    }

    fn insert_item(&mut self, slot: u32, stack: ItemStack) -> ItemStack {
        let mut slots = self.slots.lock().unwrap();
        let idx = slot as usize;
        if idx >= slots.len() || stack.is_empty() {
            return stack;
        }

        let target = &mut slots[idx];
        if target.is_empty() {
            let to_insert = stack.count().min(self.max_stack_size);
            *target = ItemStack::new(stack.item().clone(), to_insert);
            if to_insert < stack.count() {
                ItemStack::new(stack.item().clone(), stack.count() - to_insert)
            } else {
                ItemStack::empty()
            }
        } else if target.item() == stack.item() {
            let space = self.max_stack_size.saturating_sub(target.count());
            let to_insert = stack.count().min(space);
            target.set_count(target.count() + to_insert);
            if to_insert < stack.count() {
                ItemStack::new(stack.item().clone(), stack.count() - to_insert)
            } else {
                ItemStack::empty()
            }
        } else {
            stack
        }
    }

    fn extract_item(&mut self, slot: u32, amount: u32) -> ItemStack {
        let mut slots = self.slots.lock().unwrap();
        let idx = slot as usize;
        if idx >= slots.len() {
            return ItemStack::empty();
        }

        let target = &mut slots[idx];
        if target.is_empty() {
            return ItemStack::empty();
        }

        let extracted = target.count().min(amount);
        let result = ItemStack::new(target.item().clone(), extracted);
        target.set_count(target.count() - extracted);
        result
    }

    fn slot_limit(&self, _slot: u32) -> u32 {
        self.max_stack_size
    }
}
