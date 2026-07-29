use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicBool};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "neorusty", version, about = "NeoRusty — Minecraft Modding Server & Toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new NeoRusty mod project
    Init {
        mod_id: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        author: Option<String>,
    },
    /// Build a NeoRusty mod into a plugin binary
    Build {
        #[arg(default_value = ".")]
        path: String,
        #[arg(short, long)]
        release: bool,
    },
    /// Start the NeoRusty server
    Run {
        #[arg(short = 'H', long, default_value = "127.0.0.1:25565")]
        host: String,
        #[arg(short = 'P', long, default_value_t = 20)]
        max_players: u32,
        #[arg(short = 'M', long, default_value = "NeoRusty Server")]
        motd: String,
        #[arg(long, default_value = "mods")]
        mods_dir: String,
        #[arg(long, default_value = "bridge/java/lib")]
        lib_dir: String,
        #[arg(long)]
        agent_jar: Option<String>,
    },
    /// List or manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
}

#[derive(Subcommand)]
enum PluginAction {
    List,
    Enable { name: String },
    Disable { name: String },
}

fn cmd_init(mod_id: &str, name: Option<&str>, author: Option<&str>) {
    let dir = Path::new(mod_id);
    if dir.exists() {
        eprintln!("Directory '{mod_id}' already exists");
        return;
    }
    let mod_name = name.unwrap_or(mod_id);
    let author_name = author.unwrap_or("unknown");
    fs::create_dir_all(dir.join("src")).expect("create src dir");
    fs::create_dir_all(dir.join("plugins")).expect("create plugins dir");
    let cargo_toml = format!(
        r#"[package]
name = "{mod_id}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
neorusty-core = {{ git = "https://github.com/neorusty/neorusty" }}
neorusty-plugin-system = {{ git = "https://github.com/neorusty/neorusty" }}
neorusty-macros = {{ git = "https://github.com/neorusty/neorusty" }}

[profile.release]
opt-level = 3
lto = true
"#
    );
    fs::write(dir.join("Cargo.toml"), cargo_toml).expect("write Cargo.toml");
    let mod_file = format!(
        r#"use neorusty_macros::neoforge_mod;
use neorusty_plugin_system::{{Context, Plugin, PluginMetadata}};

struct {mod_id_camel}Plugin;

impl Plugin for {mod_id_camel}Plugin {{
    fn metadata(&self) -> PluginMetadata {{
        PluginMetadata {{
            id: "{mod_id}",
            name: "{mod_name}",
            version: "0.1.0",
            authors: &["{author_name}"],
            description: "{mod_name} mod for NeoRusty",
        }}
    }}

    fn on_load(&self, ctx: &Context) {{
        println!("Loading {mod_name} v0.1.0");
    }}
}}

#[neoforge_mod]
fn create_plugin() -> Box<dyn Plugin> {{
    Box::new({mod_id_camel}Plugin)
}}
"#,
        mod_id_camel = mod_id
    );
    fs::write(dir.join("src/lib.rs"), mod_file).expect("write src/lib.rs");
    let gitignore = "target/\nplugins/\n";
    fs::write(dir.join(".gitignore"), gitignore).expect("write .gitignore");
    println!("✓ Created mod project '{mod_id}' in ./{mod_id}");
    println!("  To build:  cd {mod_id} && neorusty build");
    println!("  Author:    {author_name}");
}

fn cmd_build(path: &str, release: bool) {
    let profile = if release { "release" } else { "debug" };
    let status = Command::new("cargo")
        .args(["build", "--lib"])
        .arg(if release { "--release" } else { "" })
        .current_dir(path)
        .status()
        .expect("cargo build failed");
    if !status.success() {
        eprintln!("Build failed");
        return;
    }
    let target_dir = Path::new(path).join("target").join(profile);
    let plugin_dest = Path::new(path).join("plugins");
    if let Ok(entries) = fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "so" || ext == "dylib" || ext == "dll" {
                    let dest = plugin_dest.join(path.file_name().unwrap());
                    fs::copy(&path, &dest).expect("copy plugin");
                    println!("  ✓ Copied {} to plugins/", path.file_name().unwrap().to_string_lossy());
                }
            }
        }
    }
    println!("✓ Build complete ({profile})");
}

fn cmd_run(host: &str, max_players: u32, motd: &str, mods_dir: &str, lib_dir: &str, agent_jar: Option<&str>) {
    let addr: std::net::SocketAddr = host.parse().unwrap_or_else(|_| {
        eprintln!("Invalid host address: {host}");
        std::process::exit(1);
    });

    use neorusty_core::event::EventBus;
    use neorusty_server_core::{Server, ServerConfig};

    // --- Resolve agent JAR ---
    let agent_jar = agent_jar.map(|s| s.to_string()).unwrap_or_else(|| {
        let in_dev = format!("{}/bridge/java/build/neorusty-agent.jar", env!("CARGO_MANIFEST_DIR"));
        if Path::new(&in_dev).exists() { in_dev }
        else { "neorusty-agent.jar".to_string() }
    });

    // --- Event bus + plugin manager ---
    let bus = Arc::new(EventBus::new());
    let plugin_manager = neorusty_plugin_system::PluginManager::new();

    // --- Game loop shutdown signal ---
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nShutdown signal received, stopping server...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    // --- Load native plugins ---
    let loaded = plugin_manager.load_plugins_from_dir(mods_dir);
    if !loaded.is_empty() {
        println!("Loaded native plugins: {}", loaded.join(", "));
    }

    // --- Initialize JVM bridge ---
    let jvm_manager = neorusty_bridge::init_bridge(&agent_jar);
    if let Ok(ref jvm) = jvm_manager {
        println!("JVM bridge initialized");

        // Add NeoForge libs to classpath via URLClassLoader at runtime
        let _ = jvm.with_env(|env| {
            let lib_dir_path = Path::new(lib_dir);
            if lib_dir_path.exists() {
                let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
                let cls_obj = env.find_class(&*cls)?;

                let add_lib_mtd = jni::strings::JNIString::new("addLibraryPath");
                let add_sig: jni::signature::RuntimeMethodSignature =
                    "(Ljava/lang/String;)V".parse().map_err(|_| {
                        jni::errors::Error::MethodNotFound {
                            name: "addLibraryPath".into(),
                            sig: "(Ljava/lang/String;)V".into(),
                        }
                    })?;
                let add_sig_ms: jni::signature::MethodSignature = (&add_sig).into();
                let path_str = env.new_string(lib_dir)?;
                let _ = env.call_static_method(&cls_obj, &*add_lib_mtd, &add_sig_ms, &[jni::objects::JValue::Object(&path_str)]);
            }
            Ok::<_, jni::errors::Error>(())
        });
    }

    // --- Create server ---
    let config = ServerConfig {
        host: addr,
        max_players,
        motd: motd.to_string(),
        ..Default::default()
    };
    let mut server = Server::new(config, bus);
    println!("Starting NeoRusty server on {}...", server.config.host);
    server.start();

    // --- Game loop ---
    let tick_rate = server.config.tick_rate_hz;
    let tick_duration = Duration::from_secs_f64(1.0 / tick_rate as f64);
    let mut last_tick = Instant::now();

    println!("Server running ({} ticks/sec). Press Ctrl+C to stop.", tick_rate);

    while running.load(Ordering::SeqCst) && server.running {
        let now = Instant::now();
        let elapsed = now.duration_since(last_tick);

        if elapsed >= tick_duration {
            last_tick = now;
            server.tick();

            // Notify JVM bridge of tick
            if let Ok(ref jvm) = jvm_manager {
                let tick = server.tick_count as i64;
                let _ = jvm.with_env(|env| {
                    let cls = jni::strings::JNIString::new("neorusty/agent/BridgeNative");
                    let cls_obj = env.find_class(&*cls)?;
                    let mtd = jni::strings::JNIString::new("nativeOnTick");
                    let sig: jni::signature::RuntimeMethodSignature =
                        "(J)V".parse().map_err(|_| {
                            jni::errors::Error::MethodNotFound {
                                name: "nativeOnTick".into(),
                                sig: "(J)V".into(),
                            }
                        })?;
                    let sig_ms: jni::signature::MethodSignature = (&sig).into();
                    env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[jni::objects::JValue::Long(tick)])?;
                    Ok::<_, jni::errors::Error>(())
                });
            }
        } else {
            std::thread::sleep(tick_duration - elapsed);
        }
    }

    // --- Shutdown ---
    println!("Stopping server...");
    server.stop();
    println!("Server stopped.");
}

fn cmd_plugin_list() {
    println!("Plugins: (not yet loaded)");
}

fn cmd_plugin_enable(name: &str) {
    println!("Enabled plugin: {name}");
}

fn cmd_plugin_disable(name: &str) {
    println!("Disabled plugin: {name}");
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { mod_id, name, author } => {
            cmd_init(&mod_id, name.as_deref(), author.as_deref());
        }
        Commands::Build { path, release } => {
            cmd_build(&path, release);
        }
        Commands::Run { host, max_players, motd, mods_dir, lib_dir, agent_jar } => {
            cmd_run(&host, max_players, &motd, &mods_dir, &lib_dir, agent_jar.as_deref());
        }
        Commands::Plugin { action } => match action {
            PluginAction::List => cmd_plugin_list(),
            PluginAction::Enable { name } => cmd_plugin_enable(&name),
            PluginAction::Disable { name } => cmd_plugin_disable(&name),
        },
    }
}
