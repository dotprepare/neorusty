# NeoRusty — Execution Plan

## Project Map

```
NeoRusty (Rust MC server + NeoForge API port)
├── core/            — EventBus, ResourceLocation, Codec, TriState
├── macros/          — #[derive(Event)], proc macros
├── api/             — Registries, capabilities, fluids, energy, items, networking
├── server-core/     — World, player, tick, lifecycle
├── plugin-system/   — Native + WASM plugin loading
├── bridge/          — JVM embedding for real Java NeoForge mods ← ACTIVE
├── neorusty-cli/    — CLI binary
└── neorusty/        — Integration test crate
```

## Test Dashboard

| Crate | Tests | Status |
|-------|-------|--------|
| neorusty (integration) | 3 | ✅ |
| neorusty-api | 12 | ✅ |
| neorusty-bridge (unit) | 2 | ✅ |
| neorusty-bridge (integration) | 5 | ✅ |
| neorusty-core | 7 | ✅ |
| neorusty-macros | 0 | ✅ |
| neorusty-plugin-system | 0 | ✅ |
| neorusty-server-core | 9 | ✅ |
| **Total** | **38** | ✅ |

## Phases

### Phase 0-7 — Core API & Plugin System ✅

| Phase | What | Tests |
|-------|------|-------|
| P0 | Workspace, ResourceLocation, Registry, Codec, EventBus | 7 core |
| P1 | Plugin trait, dual loaders, plugin_impl macro | — |
| P2 | Event dispatch, hooks, cancellable, parallel | — |
| P3 | DeferredRegister, DataMapLoader, snapshots | — |
| P4 | Capabilities, fluids, energy, items | 12 api |
| P5 | Networking payloads, codecs, packet dispatch | — |
| P6 | Server lifecycle, world, player, tick | 9 server-core |
| P7 | Build toolchain | — |

### Phase 8 — JVM Bridge (Done)

Bridge for loading real Java NeoForge 1.21.1 mods.

| Sub-phase | What | Status |
|-----------|------|--------|
| 8.1 | JVM embedding — `JvmManager`, `JvmConfig`, `JvmWorker` | ✅ |
| 8.2 | Java agent JAR, native method registration, init round-trip | ✅ |
| 8.3 | Registry harvest — mod registrations forwarded to Rust | ✅ |
| 8.4 | Event bridge — bidirectional game events | ✅ |
| 8.5 | World bridge — chunk/block/entity access from Rust | ✅ |
| 8.6 | Real NeoForge dependency download + classpath + bootstrap hook | ✅ |

### Phase 9 — CLI Packaging (Done)

### Phase 10 — Release, docs, examples (Done)

| Phase | What | Status |
|-------|------|--------|
| 9 | CLI packaging, game loop, mod loading | ✅ |
| 10 | Release, docs, examples | ✅ |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Rust Server (game loop)                    │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Server  │  │  PluginMgr   │  │  JVM Bridge           │  │
│  │ (tick,   │  │  (load native│  │  (JvmManager)         │  │
│  │  events) │  │   .so/.wasm) │  │  ┌────────────────┐  │  │
│  └────┬─────┘  └──────┬───────┘  │  │ RegistryStore   │  │  │
│       │               │          │  │ WorldStore      │  │  │
│       └───────┬───────┘          │  │ EventStore      │  │  │
│               │                  │  └────────────────┘  │  │
│         Arc<EventBus>            └──────────┬────────────┘  │
└─────────────────────────────────────────────┼───────────────┘
                                              │ JNI call_static
                                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Java (JVM) — neorusty-agent.jar + NeoForge + Minecraft     │
│  ┌────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │BridgeNative│ │Registry  │ │Event     │ │World         │ │
│  │ native     │ │Bridge    │ │Bridge    │ │Bridge        │ │
│  │ callbacks  │ │ simulate │ │ simulate │ │ simulate     │ │
│  └────────────┘ └──────────┘ └──────────┘ └──────────────┘ │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ NeoForgeBootstrap (tryHook → RegisterEvent listener) │  │
│  └──────────────────────────────────────────────────────┘  │
│  bridge/java/lib/ (114 jars: NeoForge + MC + deps)        │
└─────────────────────────────────────────────────────────────┘
```

## Bridge Files

| File | Role |
|------|------|
| `bridge/src/jvm.rs` | `JvmManager`, `JvmConfig`, start/attach/call |
| `bridge/src/native.rs` | Native method registration + implementations |
| `bridge/src/worker.rs` | `JvmWorker` — async command queue |
| `bridge/src/registry.rs` | `RegistryEntry`, global `RegistryStore` |
| `bridge/src/lib.rs` | `init_bridge()` convenience |
| `bridge/java/.../NeoRustyAgent.java` | Agent entry, init, status |
| `bridge/java/.../BridgeNative.java` | `native` method declarations |
| `bridge/java/.../RegistryBridge.java` | Registry forwarding + simulation |
| `bridge/java/.../EventBridge.java` | Event forwarding + simulation |
| `bridge/src/world.rs` | `WorldStore`, `BlockPos`, `ChunkData`, global `WORLD` |
| `bridge/java/.../WorldBridge.java` | World state simulation + forwarding |
| `bridge/java/.../NeoForgeBootstrap.java` | NeoForge registry hook + fallback |
| `bridge/java/build/neorusty-agent.jar` | Compiled agent JAR |
| `bridge/java/lib/` | NeoForge + Minecraft dep jars (128 MB, 114 files) |
| `bridge/scripts/download_deps.py` | Dependency downloader |
| `neorusty-cli/src/main.rs` | CLI binary: init, build, run, plugin commands |
| `neorusty-cli/Cargo.toml` | CLI deps (clap, bridge, ctrlc, jni) |

## Next Steps

_(All phases complete — ready for use)_
