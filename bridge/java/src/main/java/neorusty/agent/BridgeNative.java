package neorusty.agent;

/**
 * JNI bridge between Rust and Java.
 * Rust implements the native methods; Java calls them for callbacks.
 * Rust calls Java methods on this class directly via JNI.
 */
public class BridgeNative {

    // Rust → Java: initialized by Rust after JVM starts
    private static boolean initialized = false;

    /**
     * Called by Rust to initialize the bridge.
     * @param neoForgeDir path to NeoForge installation
     * @return true if successful
     */
    public static native boolean nativeOnInit(String neoForgeDir, String modsDir);

    /**
     * Called by Rust to dispatch a game event to Java.
     * @param eventType event type identifier
     * @param eventData serialized event data
     */
    public static native void nativeOnEvent(String eventType, byte[] eventData);

    /**
     * Called by Rust every game tick.
     * @param tick current tick number
     */
    public static native void nativeOnTick(long tick);

    // Java → Rust: called by agent to forward registrations to Rust
    static native void nativeRegisterBlock(String id, byte[] data);
    static native void nativeRegisterItem(String id, byte[] data);
    static native void nativeRegisterFluid(String id, byte[] data);
    static native void nativeRegisterEntity(String id, byte[] data);
    static native void nativeRegisterBlockEntity(String id, byte[] data);

    // Java → Rust: called by agent to forward events
    static native void nativeFireEvent(String eventType, byte[] eventData);

    // Java → Rust: logging
    static native void nativeLog(int level, String message);

    // Java → Rust: world state
    static native void nativeWorldSetBlock(int x, int y, int z, String blockId, byte[] data);
    static native void nativeWorldRemoveBlock(int x, int y, int z);
    static native int nativeWorldGetBlockCount();

    public static boolean isInitialized() {
        return initialized;
    }

    public static void setInitialized(boolean v) {
        initialized = v;
    }
}
