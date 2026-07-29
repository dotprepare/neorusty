package neorusty.agent;

import java.lang.instrument.Instrumentation;

/**
 * JVM agent entry point. Loaded via -javaagent:neorusty-agent.jar.
 * Instrumentation is stored for class transformation if needed.
 */
public class NeoRustyAgent {

    private static Instrumentation instrumentation;
    private static RegistryBridge registryBridge;
    private static EventBridge eventBridge;
    private static WorldBridge worldBridge;

    public static void premain(String args, Instrumentation inst) {
        instrumentation = inst;
        System.out.println("[NeoRusty] Agent loaded");
        System.out.println("[NeoRusty] Java version: " + System.getProperty("java.version"));
    }

    public static void agentmain(String args, Instrumentation inst) {
        instrumentation = inst;
    }

    public static Instrumentation getInstrumentation() {
        return instrumentation;
    }

    /**
     * Initialize the bridge. Called by Rust after JVM starts.
     * This is a regular (non-native) method that Rust calls via JNI call_static.
     */
    public static boolean init(String neoForgeDir, String modsDir) {
        if (BridgeNative.isInitialized()) {
            System.out.println("[NeoRusty] Bridge already initialized");
            return true;
        }

        System.out.println("[NeoRusty] Initializing bridge...");
        System.out.println("[NeoRusty]   NeoForge dir: " + neoForgeDir);
        System.out.println("[NeoRusty]   Mods dir: " + modsDir);

        registryBridge = new RegistryBridge();
        eventBridge = new EventBridge();
        worldBridge = new WorldBridge();

        // Try NeoForge bootstrap
        boolean neoForgeHooked = NeoForgeBootstrap.tryHook();
        BridgeNative.nativeLog(1, "NeoForge bootstrap: " + (neoForgeHooked ? "hooked" : "fallback mode"));

        BridgeNative.setInitialized(true);

        // In future: bootstrap NeoForge, hook registries, etc.
        BridgeNative.nativeOnInit(neoForgeDir, modsDir);
        BridgeNative.nativeLog(1, "Bridge initialized successfully");

        return true;
    }

    /**
     * Simulate registry harvest (for testing without real NeoForge).
     */
    public static void simulateRegistrations() {
        if (registryBridge == null) {
            BridgeNative.nativeLog(0, "RegistryBridge not initialized");
            return;
        }
        registryBridge.simulateRegistrations();
    }

    /**
     * Simulate bidirectional event traffic (for testing).
     */
    public static void simulateEvents() {
        if (eventBridge == null) {
            BridgeNative.nativeLog(0, "EventBridge not initialized");
            return;
        }
        eventBridge.simulateEvents();
    }

    /**
     * Simulate world state (for testing without real NeoForge).
     */
    public static void simulateWorld() {
        if (worldBridge == null) {
            BridgeNative.nativeLog(0, "WorldBridge not initialized");
            return;
        }
        worldBridge.simulateWorld();
    }

    /**
     * Fire an event from Rust into Java.
     */
    public static void fireEvent(String eventType, byte[] eventData) {
        if (eventBridge == null) {
            BridgeNative.nativeLog(0, "EventBridge not initialized");
            return;
        }
        eventBridge.fireFromRust(eventType, eventData);
    }

    /**
     * Check if NeoForge hooks are active.
     */
    public static boolean isNeoForgeHooked() {
        return NeoForgeBootstrap.isHooked();
    }

    /**
     * Check if NeoForge classes are loadable.
     */
    public static boolean canLoadNeoForgeClasses() {
        return NeoForgeBootstrap.canLoadNeoForgeClasses();
    }

    /**
     * Get list of missing NeoForge classes.
     */
    public static String[] getMissingNeoForgeClasses() {
        return NeoForgeBootstrap.getMissingClasses();
    }

    /**
     * Called by Rust every game tick.
     */
    public static void onTick(long tick) {
        BridgeNative.nativeOnTick(tick);
    }

    /**
     * Add a directory of jar files to the system class loader's classpath.
     * Called by Rust at startup to load NeoForge dependencies from lib/.
     */
    public static void addLibraryPath(String dirPath) {
        try {
            java.io.File dir = new java.io.File(dirPath);
            if (!dir.isDirectory()) return;
            java.net.URLClassLoader sysloader = (java.net.URLClassLoader) ClassLoader.getSystemClassLoader();
            java.lang.reflect.Method addURL = java.net.URLClassLoader.class.getDeclaredMethod("addURL", java.net.URL.class);
            addURL.setAccessible(true);
            for (java.io.File jar : dir.listFiles((d, name) -> name.endsWith(".jar"))) {
                addURL.invoke(sysloader, jar.toURI().toURL());
            }
        } catch (Exception e) {
            BridgeNative.nativeLog(0, "addLibraryPath failed: " + e.getMessage());
        }
    }

    /**
     * Get a summary of the bridge state.
     */
    public static String getStatus() {
        return "NeoRusty Agent | Java: " + System.getProperty("java.version")
            + " | Initialized: " + BridgeNative.isInitialized()
            + " | NeoForge: " + (NeoForgeBootstrap.isHooked() ? "hooked" : "fallback");
    }
}
