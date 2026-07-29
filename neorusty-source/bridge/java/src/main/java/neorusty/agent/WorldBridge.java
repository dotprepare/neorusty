package neorusty.agent;

/**
 * Bridge for world state (chunks, blocks, entities).
 * Java pushes world state to Rust via native methods.
 * Rust queries world state via the WorldStore on the Rust side.
 */
public class WorldBridge {

    /**
     * Simulate world state for testing.
     * Places blocks across multiple chunks.
     */
    public void simulateWorld() {
        BridgeNative.nativeLog(1, "Simulating world state...");

        // Chunk (0, 0) — some surface blocks
        placeBlock(0, 64, 0, "minecraft:grass_block", new byte[]{0});
        placeBlock(1, 64, 0, "minecraft:dirt", new byte[]{0});
        placeBlock(0, 64, 1, "minecraft:grass_block", new byte[]{0});
        placeBlock(2, 64, 0, "minecraft:stone", new byte[]{0});
        placeBlock(0, 63, 0, "minecraft:stone", new byte[]{0});

        // Chunk (1, 0) — more blocks
        placeBlock(16, 64, 0, "minecraft:oak_log", new byte[]{0});
        placeBlock(16, 65, 0, "minecraft:oak_leaves", new byte[]{0});
        placeBlock(16, 64, 1, "minecraft:oak_log", new byte[]{0});

        // Chunk (0, 1)
        placeBlock(0, 64, 16, "minecraft:water", new byte[]{0});
        placeBlock(1, 64, 16, "minecraft:water", new byte[]{0});

        BridgeNative.nativeLog(1, "Simulated " + BridgeNative.nativeWorldGetBlockCount() + " world blocks");
    }

    private void placeBlock(int x, int y, int z, String blockId, byte[] data) {
        BridgeNative.nativeWorldSetBlock(x, y, z, blockId, data);
    }
}
