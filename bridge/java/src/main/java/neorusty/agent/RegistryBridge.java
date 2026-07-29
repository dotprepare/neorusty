package neorusty.agent;

import java.util.*;

/**
 * Bridges NeoForge registry events to Rust-native registries.
 * Collects all mod registrations at init time and forwards them via JNI.
 */
public class RegistryBridge {

    private final Map<String, List<String>> pendingRegistrations = new LinkedHashMap<>();

    public RegistryBridge() {
        System.out.println("[NeoRusty] RegistryBridge created");
    }

    /**
     * Register a block and forward to Rust.
     */
    public void registerBlock(String id, byte[] data) {
        pendingRegistrations.computeIfAbsent("block", k -> new ArrayList<>()).add(id);
        BridgeNative.nativeRegisterBlock(id, data);
    }

    /**
     * Register an item and forward to Rust.
     */
    public void registerItem(String id, byte[] data) {
        pendingRegistrations.computeIfAbsent("item", k -> new ArrayList<>()).add(id);
        BridgeNative.nativeRegisterItem(id, data);
    }

    /**
     * Register a fluid and forward to Rust.
     */
    public void registerFluid(String id, byte[] data) {
        pendingRegistrations.computeIfAbsent("fluid", k -> new ArrayList<>()).add(id);
        BridgeNative.nativeRegisterFluid(id, data);
    }

    /**
     * Simulate NeoForge registrations for testing the bridge.
     * Creates mock blocks, items, fluids, entities, and block entities.
     */
    public void simulateRegistrations() {
        System.out.println("[NeoRusty] Simulating registry harvest...");

        // Mock blocks
        registerBlock("minecraft:stone", "stone".getBytes());
        registerBlock("minecraft:dirt", "dirt".getBytes());
        registerBlock("minecraft:grass_block", "grass".getBytes());

        // Mock items
        registerItem("minecraft:diamond", "diamond".getBytes());
        registerItem("minecraft:stick", "stick".getBytes());

        // Mock fluids
        registerFluid("minecraft:water", "water".getBytes());
        registerFluid("minecraft:lava", "lava".getBytes());

        // Mock entities (via nativeRegisterEntity)
        BridgeNative.nativeRegisterEntity("minecraft:zombie", "zombie".getBytes());
        BridgeNative.nativeRegisterEntity("minecraft:skeleton", "skeleton".getBytes());
        BridgeNative.nativeRegisterEntity("minecraft:creeper", "creeper".getBytes());

        // Mock block entities (via nativeRegisterBlockEntity)
        BridgeNative.nativeRegisterBlockEntity("minecraft:furnace", "furnace".getBytes());
        BridgeNative.nativeRegisterBlockEntity("minecraft:chest", "chest".getBytes());

        System.out.println("[NeoRusty] Simulated " + totalCount() + " registrations");
    }

    public Map<String, List<String>> getPendingRegistrations() {
        return pendingRegistrations;
    }

    public int totalCount() {
        return pendingRegistrations.values().stream().mapToInt(List::size).sum();
    }
}
