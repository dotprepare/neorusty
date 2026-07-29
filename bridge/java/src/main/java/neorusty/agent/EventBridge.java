package neorusty.agent;

import java.util.concurrent.ConcurrentHashMap;
import java.util.Map;

/**
 * Bridges game events between Rust and Java bidirectionally.
 * For MVP: a simple registry of event handlers and a dispatch mechanism.
 */
public class EventBridge {

    private final ConcurrentHashMap<String, Integer> eventCounts = new ConcurrentHashMap<>();

    public EventBridge() {
        System.out.println("[NeoRusty] EventBridge created");
    }

    /**
     * Fire an event from Rust into Java.
     * In future: wraps as NeoForge event and posts to MinecraftForge.EVENT_BUS.
     * Also notifies Rust via nativeOnEvent so Rust can track dispatch.
     */
    public void fireFromRust(String eventType, byte[] eventData) {
        eventCounts.merge(eventType, 1, Integer::sum);
        BridgeNative.nativeLog(2, "Event Rust→Java: " + eventType);
        BridgeNative.nativeOnEvent(eventType, eventData);
    }

    /**
     * Fire an event from Java back to Rust.
     */
    public void fireToRust(String eventType, byte[] eventData) {
        eventCounts.merge(eventType, 1, Integer::sum);
        BridgeNative.nativeFireEvent(eventType, eventData);
    }

    /**
     * Simulate bidirectional event traffic for testing.
     * Fires 3 events Java→Rust and 2 events Rust→Java.
     */
    public void simulateEvents() {
        System.out.println("[NeoRusty] Simulating bidirectional events...");

        // Java → Rust events
        fireToRust("tick_start", "tick_42".getBytes());
        fireToRust("entity_damage", "zombie:10".getBytes());
        fireToRust("block_break", "stone".getBytes());

        // Rust → Java events (via nativeOnEvent callback)
        BridgeNative.nativeOnEvent("server_start", "starting".getBytes());
        BridgeNative.nativeOnEvent("player_join", "player1".getBytes());

        System.out.println("[NeoRusty] Simulated " + getTotalCount() + " events");
    }

    public int getEventCount(String eventType) {
        return eventCounts.getOrDefault(eventType, 0);
    }

    public int getTotalCount() {
        return eventCounts.values().stream().mapToInt(Integer::intValue).sum();
    }

    public Map<String, Integer> getAllCounts() {
        return new ConcurrentHashMap<>(eventCounts);
    }
}
