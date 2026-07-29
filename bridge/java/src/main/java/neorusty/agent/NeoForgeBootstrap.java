package neorusty.agent;

/**
 * Bootstraps real NeoForge registry event hooking.
 * When NeoForge is on the classpath, subscribes to the mod bus
 * and forwards registrations to Rust via native methods.
 * Falls back gracefully if NeoForge isn't available.
 */
public class NeoForgeBootstrap {

    private static boolean hooked = false;

    /**
     * Try to hook into NeoForge's mod event bus.
     * Returns true if hooks were successfully registered.
     */
    public static boolean tryHook() {
        if (hooked) {
            return true;
        }
        // Attempt to load NeoForge classes and subscribe to registry events
        try {
            Class.forName("net.neoforged.bus.api.IEventBus");
            Class.forName("net.neoforged.neoforge.registries.RegisterEvent");
            hooked = doHook();
            if (hooked) {
                BridgeNative.nativeLog(1, "NeoForge hooks registered successfully");
            }
            return hooked;
        } catch (ClassNotFoundException e) {
            BridgeNative.nativeLog(2, "NeoForge not on classpath: " + e.getMessage());
            return false;
        } catch (Throwable t) {
            BridgeNative.nativeLog(0, "NeoForge hook failed: " + t.getMessage());
            return false;
        }
    }

    /**
     * Check if hooks are active.
     */
    public static boolean isHooked() {
        return hooked;
    }

    /**
     * Test that the necessary NeoForge classes can be loaded.
     * Does not attempt full bootstrap.
     */
    public static boolean canLoadNeoForgeClasses() {
        try {
            Class.forName("net.neoforged.bus.api.IEventBus");
            Class.forName("net.neoforged.neoforge.registries.RegisterEvent");
            Class.forName("net.neoforged.fml.ModLoadingContext");
            Class.forName("net.neoforged.fml.event.lifecycle.FMLCommonSetupEvent");
            CoreLibsCheck.load();
            return true;
        } catch (Throwable e) {
            return false;
        }
    }

    /**
     * List which NeoForge classes are missing.
     */
    public static String[] getMissingClasses() {
        String[] classes = {
            "net.neoforged.bus.api.IEventBus",
            "net.neoforged.neoforge.registries.RegisterEvent",
            "net.neoforged.fml.ModLoadingContext",
            "net.neoforged.fml.event.lifecycle.FMLCommonSetupEvent",
        };
        java.util.ArrayList<String> missing = new java.util.ArrayList<>();
        for (String cls : classes) {
            try {
                Class.forName(cls);
            } catch (ClassNotFoundException e) {
                missing.add(cls);
            }
        }
        return missing.toArray(new String[0]);
    }

    private static boolean doHook() {
        BridgeNative.nativeLog(1, "Attempting NeoForge event bus hook...");
        try {
            // Resolve the mod event bus via reflection
            Class<?> mlc = Class.forName("net.neoforged.fml.ModLoadingContext");
            java.lang.reflect.Method get = mlc.getMethod("get");
            Object context = get.invoke(null);
            java.lang.reflect.Method getBus = mlc.getMethod("getModEventBus");
            Object modBus = getBus.invoke(context);

            // Subscribe to RegisterEvent
            if (modBus != null) {
                Object listener = createRegisterListener();
                Class<?> busClass = modBus.getClass();
                busClass.getMethod("addListener", Object.class)
                       .invoke(modBus, listener);
                BridgeNative.nativeLog(1, "Subscribed to NeoForge RegisterEvent");
                return true;
            }
        } catch (Exception e) {
            BridgeNative.nativeLog(0, "Hook failed: " + e.getMessage());
        }
        return false;
    }

    private static Object createRegisterListener() {
        return new Object() {
            @SuppressWarnings("unused")
            public void onRegister(Object event) {
                try {
                    Class<?> eventClass = event.getClass();
                    java.lang.reflect.Method getRegistryKey = eventClass.getMethod("getRegistryKey");
                    Object registryKey = getRegistryKey.invoke(event);
                    String registryName = registryKey.toString();

                    BridgeNative.nativeLog(2, "Registry event: " + registryName);

                    java.lang.reflect.Method getEntries = eventClass.getMethod("getEntries");
                    Iterable<?> entries = (Iterable<?>) getEntries.invoke(event);
                    for (Object entry : entries) {
                        String id = entry.toString();
                        BridgeNative.nativeLog(2, "  Entry: " + id);
                        // Forward to appropriate native method based on registry
                        String type = classifyRegistry(registryName);
                        if (type != null) {
                            forwardRegistration(type, id);
                        }
                    }
                } catch (Exception e) {
                    BridgeNative.nativeLog(0, "RegisterEvent handler error: " + e.getMessage());
                }
            }
        };
    }

    private static String classifyRegistry(String registryName) {
        if (registryName.contains("block")) return "block";
        if (registryName.contains("item")) return "item";
        if (registryName.contains("fluid")) return "fluid";
        if (registryName.contains("entity_type")) return "entity";
        if (registryName.contains("block_entity")) return "block_entity";
        return null;
    }

    private static void forwardRegistration(String type, String id) {
        byte[] data = id.getBytes(java.nio.charset.StandardCharsets.UTF_8);
        switch (type) {
            case "block" -> BridgeNative.nativeRegisterBlock(id, data);
            case "item" -> BridgeNative.nativeRegisterItem(id, data);
            case "fluid" -> BridgeNative.nativeRegisterFluid(id, data);
            case "entity" -> BridgeNative.nativeRegisterEntity(id, data);
            case "block_entity" -> BridgeNative.nativeRegisterBlockEntity(id, data);
        }
    }

    /**
     * Inner class to verify core library classes at load time.
     */
    private static class CoreLibsCheck {
        static void load() throws ClassNotFoundException {
            Class.forName("com.google.gson.Gson");
            Class.forName("com.mojang.brigadier.CommandDispatcher");
            Class.forName("org.apache.logging.log4j.Logger");
        }
    }
}
