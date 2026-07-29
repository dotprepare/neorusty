use neorusty_core::event::EventPriority;
use neorusty_macros::neoforge_mod;
use neorusty_plugin_system::{Context, Plugin, PluginMetadata};
use neorusty_plugin_system::loader::PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub static NEORUSTY_API_VERSION: u32 = PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub static NEORUSTY_METADATA: PluginMetadata = PluginMetadata {
    id: "example_mod",
    name: "Example Mod",
    version: "0.1.0",
    authors: &["NeoRusty Contributors"],
    description: "Example NeoRusty mod demonstrating the plugin system",
};

#[derive(Debug, Clone)]
struct ExampleMod;

impl Plugin for ExampleMod {
    fn metadata(&self) -> PluginMetadata {
        NEORUSTY_METADATA.clone()
    }

    fn on_load(&self, ctx: &Context) {
        println!("ExampleMod v{} loaded!", self.metadata().version);

        ctx.on::<GreetEvent>(EventPriority::Normal, |e: &GreetEvent| {
            println!("[ExampleMod] Hello, {}!", e.who);
            GreetEvent { who: e.who.clone() }
        });
    }
}

#[derive(Debug, Clone)]
struct GreetEvent {
    who: String,
}

impl neorusty_core::Event for GreetEvent {}

#[neoforge_mod]
fn create_plugin() -> Box<dyn Plugin> {
    Box::new(ExampleMod)
}
