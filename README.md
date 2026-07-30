# NeoRusty

A NeoForge-compatible Minecraft mod loader rewritten in Rust, built on top of PumpkinMC.

> [!WARNING]
> NeoRusty is currently **experimental**.
>
> Development is closely tied to PumpkinMC. Until PumpkinMC reaches a stable 1.0 release, NeoRusty will remain under active development and APIs may change without notice.

## Goals

- Provide a Rust-native implementation of the NeoForge mod loading system.
- Support existing NeoForge Java mods where possible.
- Enable high-performance Rust mods.
- Take advantage of PumpkinMC's multithreaded server architecture.

## Features

### Current
- [x] Minecraft server (powered by PumpkinMC)
- [x] Multithreaded server architecture (via PumpkinMC)
- [x] Native Rust mod API
- [ ] 

### Planned
- [ ] NeoForge-compatible mod loading
- [ ] Java NeoForge mod compatibility layer
- [ ] Automatic mod metadata conversion
- [ ] Dependency resolution
- [ ] Mixins / bytecode compatibility (if feasible)
- [ ] Configuration system
- [ ] Event bus
- [ ] Networking API
- [ ] Data pack integration

## Roadmap

### Phase 1
- [x] Integrate PumpkinMC
- [x] Basic mod discovery
- [x] Load Rust mods

### Phase 2
- [ ] NeoForge metadata parser
- [ ] Dependency resolution
- [ ] Event system

### Phase 3
- [ ] Java mod compatibility
- [ ] Plugin ecosystem
- [ ] Documentation

## Status

| Component | Status |
|-----------|--------|
| Server | Working |
| Mod Loader |  In Progress |
| Rust Mods |  Planned |
| Java Mod Support |  Planned |
| Documentation |  In Progress |

## License

LGPL 2.0 (Follow NeoForge)
