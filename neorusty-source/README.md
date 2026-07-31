# NeoRusty

**Minecraft modding server — native Rust plugins with a JVM bridge for NeoForge.**

```
neorusty run -H 0.0.0.0:25565 -P 20 -M "My NeoRusty Server"
```

## Architecture

| Layer | Crate | Purpose |
|-------|-------|---------|
| **Core** | `neorusty-core` | Event system, abstract traits |
| **API** | `neorusty-api` | Registry, item, fluid, capability, energy, network |
| **Plugin System** | `neorusty-plugin-system` | Plugin trait, native/WASM loaders, lifecycle |
| **Macros** | `neorusty-macros` | `#[neoforge_mod]`, `#[event_handler]`, `#[derive(Event)]` |
| **Server** | `neorusty-server-core` | Server loop (PumpkinMC engine), config, lifecycle events |
| **CLI** | `neorusty-cli` | Binary — `init`, `build`, `run`, `plugin` commands |
| **Bridge** | `neorusty-bridge` | JVM invocation, registry/world/event stores, NeoForge bootstrap |

## Quick Start

```bash
# Start a server
cargo run -- run

# Scaffold a new mod project
cargo run -- init my_mod --name "My Mod" --author "You"

# Build a mod (cdylib plugin)
cd my_mod && cargo build --lib --release
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `init <id>` | Scaffold a new mod project |
| `build` | Build native plugin from the project |
| `run` | Start the server with JVM bridge |
| `plugin list` | List loaded plugins |
| `plugin enable <name>` | Enable a plugin |
| `plugin disable <name>` | Disable a plugin |

### Run Options

```
-H, --host         Server address          [default: 127.0.0.1:25565]
-P, --max-players  Max players             [default: 20]
-M, --motd         Server MOTD             [default: NeoRusty Server]
--mods-dir         Native plugin directory [default: mods]
--lib-dir          Java library directory  [default: bridge/java/lib]
--agent-jar        JVM agent JAR path      [auto]
```

## Writing a Mod

```rust
use neorusty_core::Event;
use neorusty_core::event::EventPriority;
use neorusty_macros::neoforge_mod;
use neorusty_plugin_system::{Context, Plugin, PluginMetadata};

struct MyPlugin;

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "my_mod",
            name: "My Mod",
            version: "0.1.0",
            authors: &["You"],
            description: "My first NeoRusty mod",
        }
    }

    fn on_load(&self, ctx: &Context) {
        println!("MyMod loaded!");

        ctx.on::<MyEvent>(EventPriority::Normal, |e: &MyEvent| {
            println!("Event: {}", e.message);
            MyEvent { message: e.message.clone() }
        });
    }
}

#[derive(Debug, Clone)]
struct MyEvent { message: String }

impl Event for MyEvent {}

#[neoforge_mod]
fn create() -> Box<dyn Plugin> {
    Box::new(MyPlugin)
}
```

Build with: `cargo build --lib --release`

## JVM Bridge

NeoRusty embeds a JVM to load real NeoForge mods. The bridge provides:

- **Registry** — items, blocks, fluids, entities harvested from NeoForge
- **Events** — bidirectional Rust↔Java event forwarding
- **World** — block placement, query, removal proxied to Java
- **Bootstrap** — automatic NeoForge class loading with graceful fallback

## Example

See [`examples/example_mod/`](./examples/example_mod/) for a complete, buildable mod.

## License

MIT
