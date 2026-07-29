# CLAUD — NeoRusty Project Context

## Identity

**NeoRusty** — Rust port of the NeoForge Minecraft modding API with a plugin system, targeting a multi-threaded Rust Minecraft server (Pumpkin MC-inspired). Supports both native Rust plugins and (planned) WASM loaders, plus a JVM bridge for real Java NeoForge mods (Phase 8).

**Epic Key:** NEO-1
**Current Version:** 0.1.0 (pre-release, own versioning — not tied to NeoForge/MC version)
**License:** LGPL v2.1 (API crate) / GPL v3 (server/core) — mirroring NeoForge x Pumpkin

---

## Repository

This repo is **standalone** — not a fork of NeoForge or Pumpkin. The NeoForge source is present in `./src/` as a reference for the JVM bridge (Phase 8), but the `.git` root belongs to NeoRusty.

- **Do NOT push** to any remote unless explicitly told.
- **Do NOT** commit NeoForge's files (they're untracked by design).
- **Patch numbering** is independent (0.1.0 → 0.2.0 → etc.).
- Commit with conversational messages.

---

## Directory Map

```
/
├── CLAUD.md              ← This file. Read first in every session.
├── SKILLS.md             ← NeoForge skill reference (for bridge work).
├── plans.md              ← Full discovery & implementation plan.
├── Cargo.toml            ← Workspace root.
│
├── core/                 ← neorusty-core: ResourceLocation, Registry, EventBus, TriState
├── neorusty-macros/      ← neorusty-macros: #[derive(Event)], #[neoforge_mod], #[event_handler]
├── plugin-system/        ← neorusty-plugin-system: Plugin trait, PluginManager, loaders
├── api/                  ← neorusty-api: DeferredRegister, Capabilities, Fluids, Energy, Items, Network
├── server-core/          ← neorusty-server-core: Server, World, Player, config, lifecycle events
├── neorusty/             ← neorusty: binary — server executable
├── neorusty-cli/         ← neorusty-cli: CLI tool (init, build, run, plugin)
│
├── bridge/               ← (planned Phase 8) neorusty-bridge + java agent
│
├── src/                  ← NeoForge 1.21.1 reference source (NOT part of NeoRusty build)
├── patches/              ← NeoForge patch system
├── projects/             ← NeoForge subprojects
├── Pumpkin-master/       ← Pumpkin MC server reference
└── .agents/skills/       ← OpenCode agent skill files
```

---

## Crate Details

### `neorusty-core` (`core/`)
Base types shared everywhere.
- `ResourceLocation { namespace, path }` — Minecraft-style identifier
- `ResourceKey<T>` — typed registry reference
- `Registry<T>` trait + `MappedRegistry<T>` (DashMap-backed, thread-safe)
- `RegistryHandle<T>` — `Reference(ResourceKey<T>) | Direct(Arc<T>)`
- `Event` trait, `HasResult`, `Cancellable` sub-traits
- `EventPriority` — `Lowest | Low | Normal | High | Highest | Monitor`
- `EventBus` — sequential + parallel (rayon) dispatch, priority-sorted
- `TriState` — `Allow | Default | Deny`
- Tests: 7/7 passing

### `neorusty-macros` (`neorusty-macros/`)
Proc-macro crate.
- `#[derive(Event)]` — implement `Event` trait on a struct
- `#[neoforge_mod]` — generate `#[no_mangle] pub extern "C" fn neorusty_plugin_entry()`
- `#[event_handler]` — pass-through attribute (marker for tooling)

### `neorusty-plugin-system` (`plugin-system/`)
Dynamic plugin loading.
- `Plugin` trait — `metadata()`, `on_load()`, `on_unload()`
- `PluginManager` — DashMap storage, `register()`, `load_plugins_from_dir()`
- `Context` — wraps `EventBus` + plugin_id, `on::<E>()` for event registration
- `PluginLoader` trait — `can_load()`, `load()`, `name()`
- `NativePluginLoader` — libloading-based, reads `NEORUSTY_API_VERSION`, `NEORUSTY_METADATA`, `neorusty_plugin_entry`
- `WasmPluginLoader` — stub (wasmtime not integrated yet)
- `PluginMetadata` — id, name, version, authors, description

### `neorusty-api` (`api/`)
Higher-level NeoForge API port.
- **Registry:** `RegistryManager` (thread-safe, freezable), `DeferredRegister<T>`, `DeferredHolder<T>`
- **Capability:** `Capability<T, C>` with `BlockCapability`, `EntityCapability`, `ItemCapability` type aliases
- **Fluids:** `FluidStack`, `FluidType` (builder pattern), `BUCKET_VOLUME`
- **Energy:** `EnergyStorage` trait, `BasicEnergyStorage` (AtomicU32)
- **Items:** `ItemStack`, `ItemHandler` trait, `BasicItemHandler` (Mutex-backed)
- **Network:** `StreamCodec<T>`, `VarIntCodec`, `StringCodec`, `UnitCodec`
  - `CustomPacketPayload` trait, `PayloadRegistrar`, `PacketDistributor`
  - `PacketFlow` (Clientbound/Serverbound/Bidirectional), `PayloadContext`
- Tests: 12/12 passing

### `neorusty-server-core` (`server-core/`)
Game server core.
- `Server` — config, worlds, players, event bus, lifecycle events, tick loop
- `World` / `WorldManager` — dimension management, ticking
- `Player` — UUID, username, game mode, health, food, dimension
- `Uuid` — custom [u8; 16] (avoids uuid crate dep)
- `GameMode`, `Difficulty` enums
- `ServerConfig` — host, max_players, motd, online_mode, tick_rate
- Lifecycle events: `ServerStartingEvent`, `ServerStartedEvent`, `ServerStoppingEvent`, `ServerTickEvent`
- Tests: 9/9 passing

### `neorusty` (`neorusty/`)
Binary crate — server executable. Thin wiring layer.

### `neorusty-cli` (`neorusty-cli/`)
CLI toolchain (clap).
- `neorusty init <mod_id>` — scaffold a mod project
- `neorusty build` — compile cdylib, copy to plugins/
- `neorusty run` — start server (optional `--host`, `--max-players`, `--motd`)
- `neorusty plugin [list|enable|disable]`

---

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Event dispatch | Sync (default) + parallel (opt-in via rayon) | Match NeoForge semantics; parallel via `post_parallel()` |
| Registry storage | `DashMap` + `RwLock` | Concurrent reads, single writer; DashMap for per-key locking |
| Capabilities | Trait-based with `TypeId` maps | Maps Java's generic `Capability<T, C>` to Rust traits |
| Energy | `AtomicU32` | Lock-free, sufficient for FE values |
| Inventory | `Mutex` per container | Safety + simplicity, not hot path |
| Plugin loading | `libloading` (native) + wasmtime (planned) | Dual approach per Pumpkin pattern |
| Networking | Custom `StreamCodec<T>` + protobuf (planned) | No serde dependency for MC protocol |
| Error handling | `String` errors for MVP | Will migrate to `thiserror` later |
| Async | `tokio` for networking only | Tick loop is sync (rayon for parallelism) |

---

## Java-to-Rust Mapping

| Java | Rust |
|---|---|
| `@SubscribeEvent` | `EventBus::register()` / `Context::on()` |
| `Class<T>` | `TypeId` / generic trait bound |
| `ResourceLocation` | `ResourceLocation` struct |
| `Registry<T>` | `MappedRegistry<T>` |
| `DeferredRegister` | `DeferredRegister<T>` |
| `Holder<T>` | `RegistryHandle<T>` |
| `Codec<T>` | `StreamCodec<T>` (future: serde) |
| `IEventBus` | `EventBus` |
| `Event.Result` | `TriState` |
| `@Nullable` | `Option<T>` |
| `NonNullList<T>` | `Vec<T>` |
| `simulate` parameter | Atomic rollover / `&mut self` |

---

## Threading Model

| Component | Strategy |
|---|---|
| Server tick | Parallel rayon work-stealing |
| Event bus | Sync on tick thread + optional parallel groups |
| Registries | `RwLock` / `DashMap` |
| Capabilities | `DashMap` per world |
| Energy | `AtomicU32` (lock-free) |
| Inventory | `Mutex` per container |
| Networking | `tokio` async tasks (planned full integration) |
| Plugin loading | Sync during init |
| Chunk I/O | `tokio::task::spawn_blocking` (planned) |
| WASM plugins | Isolated wasmtime instances (planned) |
| JVM bridge | Dedicated worker thread + mpsc command queue (Phase 8) |

---

## Phase Plan & Progress

| Phase | Status | What |
|---|---|---|
| P0 | ✓ | Core foundation: workspace, ResourceLocation, Registry, EventBus |
| P1 | ✓ | Macros: `#[derive(Event)]`, `#[neoforge_mod]` |
| P2 | ✓ | Plugin system: Plugin trait, PluginManager, NativePluginLoader, WASM stub |
| P3 | ✓ | API: DeferredRegister, RegistryManager, Capabilities, Fluids, Energy, Items |
| P4 | ✓ | Parallel event dispatch (rayon) — 7 core tests |
| P5 | ✓ | Networking: StreamCodec, PayloadRegistrar, PacketDistributor — 12 api tests |
| P6 | ✓ | Server core: Server, World, Player, lifecycle events — 9 server tests |
| P7 | ✓ | CLI: init, build, run, plugin subcommands |
| P8 | **NEXT** | **JVM bridge for real Java NeoForge mods** |

**Total tests:** 31 passing (7 core + 12 api + 9 server-core + 3 integration)
**Run:** `cargo test --workspace` (after `source ~/.cargo/env`)

---

## Phase 8 Plan (Next Up)

### Goal
Load real Java NeoForge 1.21.1 mods by embedding a JVM. Full runtime bridge.

### Architecture
- **Rust side:** `neorusty-bridge` crate — JVM manager, worker thread (mpsc command queue), type converters, registry harvester, event forwarder, world/entity bridge
- **Java side:** `neorusty-agent.jar` — Gradle project, NeoForge mod that runs inside the embedded JVM, hooks registries + events, exposes JNI/FFM callbacks
- **Protocol:** Protobuf for typed bridge messages

### Sub-phases
| Sub-phase | What |
|---|---|
| 8.1 | JVM embedding foundation — `JvmManager`, `JvmWorker`, start/stop JVM |
| 8.2 | Java agent — `neorusty-agent.jar` with RegistryBridge, EventBridge, NativeCallbacks |
| 8.3 | Registry harvest & sync — capture Java mod registrations into Rust registries |
| 8.4 | Bidirectional event bridge — Rust→Java + Java→Rust event forwarding |
| 8.5 | World & entity bridge — sync chunk data, entity ticking in JVM |
| 8.6 | Integration — wire into Server, CLI flags, startup flow |

### Key Dependencies
- `jni` crate (JNI bindings)
- `prost` / `prost-build` (protobuf)
- `libloading` (libjvm discovery — already a dep)
- JDK 21+ / Gradle (for Java agent build)
- NeoForge 1.21.1 (in `./src/`, `./projects/`)

---

## Commands

```sh
# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# Test a specific crate
cargo test -p neorusty-core
cargo test -p neorusty-api
cargo test -p neorusty-server-core

# Run the server
cargo run -p neorusty

# Use the CLI
cargo run -p neorusty-cli -- --help
cargo run -p neorusty-cli -- init mymod
cargo run -p neorusty-cli -- build
cargo run -p neorusty-cli -- run
```

---

## Conventions

- **No comments** in code unless necessary (the user's preference)
- **Crate naming:** `neorusty-*` for all crates
- **Module naming:** lowercase with underscores
- **Public API:** re-exported via `pub use` in `lib.rs`
- **Tests:** integration tests in `tests/` dir, unit tests in `#[cfg(test)] mod tests`
- **Proc macros:** live in `neorusty-macros`, imported with `use neorusty_macros::*`
- **Versioning:** independent 0.x.y (not tied to NeoForge or MC version)
