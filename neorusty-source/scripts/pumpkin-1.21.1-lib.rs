// Embeddable library entry for the pinned Pumpkin 1.21.1 base (91ae85ef).
// The upstream commit is binary-only; this mirrors main.rs's module list so the
// crate can be used as a library dependency by neorusty-server-core.
pub mod client;
pub mod commands;
pub mod entity;
pub mod error;
pub mod proxy;
pub mod rcon;
pub mod server;
pub mod world;

// `proxy::bungeecord` references `Client` from the crate root, which used to be
// provided by main.rs's `use client::Client;`.
pub use client::Client;
