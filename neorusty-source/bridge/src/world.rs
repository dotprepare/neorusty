use std::sync::{LazyLock, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone)]
pub struct BlockData {
    pub position: BlockPos,
    pub block_id: String,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ChunkData {
    pub cx: i32,
    pub cz: i32,
    pub blocks: Vec<BlockData>,
}

struct WorldStore {
    blocks: HashMap<(i32, i32, i32), BlockData>,
    chunks: HashMap<(i32, i32), ChunkData>,
}

impl Default for WorldStore {
    fn default() -> Self {
        Self {
            blocks: HashMap::new(),
            chunks: HashMap::new(),
        }
    }
}

impl WorldStore {

    fn set_block(&mut self, position: BlockPos, block_id: String, metadata: Vec<u8>) {
        let key = (position.x, position.y, position.z);
        self.blocks.insert(key, BlockData { position, block_id, metadata });
    }

    fn remove_block(&mut self, x: i32, y: i32, z: i32) {
        self.blocks.remove(&(x, y, z));
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockData> {
        self.blocks.get(&(x, y, z)).cloned()
    }

    fn block_count(&self) -> usize {
        self.blocks.len()
    }

    fn all_blocks(&self) -> Vec<BlockData> {
        self.blocks.values().cloned().collect()
    }

    fn update_chunk(&mut self, chunk: ChunkData) {
        self.chunks.insert((chunk.cx, chunk.cz), chunk.clone());
        for block in &chunk.blocks {
            let key = (block.position.x, block.position.y, block.position.z);
            self.blocks.insert(key, block.clone());
        }
    }

    fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    fn clear(&mut self) {
        self.blocks.clear();
        self.chunks.clear();
    }
}

static WORLD: LazyLock<Mutex<WorldStore>> = LazyLock::new(|| Mutex::new(WorldStore::default()));

pub fn set_block(x: i32, y: i32, z: i32, block_id: &str, metadata: Vec<u8>) {
    if let Ok(mut store) = WORLD.lock() {
        store.set_block(BlockPos::new(x, y, z), block_id.to_string(), metadata);
    }
}

pub fn remove_block(x: i32, y: i32, z: i32) {
    if let Ok(mut store) = WORLD.lock() {
        store.remove_block(x, y, z);
    }
}

pub fn get_block(x: i32, y: i32, z: i32) -> Option<BlockData> {
    WORLD
        .lock()
        .ok()
        .and_then(|store| store.get_block(x, y, z))
}

pub fn block_count() -> usize {
    WORLD
        .lock()
        .map(|store| store.block_count())
        .unwrap_or(0)
}

pub fn all_blocks() -> Vec<BlockData> {
    WORLD
        .lock()
        .map(|store| store.all_blocks())
        .unwrap_or_default()
}

pub fn update_chunk(chunk: ChunkData) {
    if let Ok(mut store) = WORLD.lock() {
        store.update_chunk(chunk);
    }
}

pub fn chunk_count() -> usize {
    WORLD
        .lock()
        .map(|store| store.chunk_count())
        .unwrap_or(0)
}

pub fn clear() {
    if let Ok(mut store) = WORLD.lock() {
        store.clear();
    }
}
