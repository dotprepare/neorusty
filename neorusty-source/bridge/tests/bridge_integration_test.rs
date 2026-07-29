use neorusty_bridge::{event, native, registry, world, jvm::{JvmConfig, JvmManager}};
use std::sync::OnceLock;

fn shared_jvm() -> &'static JvmManager {
    static JVM: OnceLock<JvmManager> = OnceLock::new();
    JVM.get_or_init(|| {
        let agent_jar = format!(
            "{}/java/build/neorusty-agent.jar",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut config = JvmConfig::default();
        config.class_path.push(agent_jar);
        // Add NeoForge lib dir if it exists
        let lib_dir = format!("{}/java/lib", env!("CARGO_MANIFEST_DIR"));
        config.add_lib_dir(&lib_dir);
        JvmManager::start_with_config(config)
            .expect("JVM should start with agent JAR")
    })
}

fn init_bridge() {
    let manager = shared_jvm();
    native::register_natives(&manager.jvm).expect("native methods should register");

    let initialized = manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("init");
        let sig: jni::signature::RuntimeMethodSignature =
            "(Ljava/lang/String;Ljava/lang/String;)Z".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "init".into(),
                    sig: "(Ljava/lang/String;Ljava/lang/String;)Z".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        let dir = env.new_string(".")?;
        let mods = env.new_string("mods")?;
        let result = env.call_static_method(
            &cls_obj, &*mtd, &sig_ms,
            &[jni::objects::JValue::Object(&dir), jni::objects::JValue::Object(&mods)],
        )?;
        Ok::<_, jni::errors::Error>(result.z()?)
    }).expect("NeoRustyAgent.init() should succeed");
    assert!(initialized, "bridge should be initialized");
}

#[test]
fn test_bridge_initialization() {
    registry::clear();
    init_bridge();

    assert!(
        native::BRIDGE_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst),
        "nativeOnInit should have been called"
    );

    let status = shared_jvm().with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("getStatus");
        let sig: jni::signature::RuntimeMethodSignature =
            "()Ljava/lang/String;".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "getStatus".into(),
                    sig: "()Ljava/lang/String;".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        let result = env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[])?;
        let obj = result.l()?;
        let jstr = unsafe {
            jni::objects::JString::from_raw(&*env, obj.into_raw() as jni::sys::jstring)
        };
        let s = jstr.try_to_string(&*env)?;
        Ok::<_, jni::errors::Error>(s)
    }).expect("getStatus should work");

    println!("Bridge status: {status}");
    assert!(status.contains("Initialized: true"), "bridge should show initialized");
}

#[test]
fn test_registry_harvest() {
    registry::clear();
    init_bridge();

    let manager = shared_jvm();
    registry::clear();

    manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("simulateRegistrations");
        let sig: jni::signature::RuntimeMethodSignature =
            "()V".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "simulateRegistrations".into(),
                    sig: "()V".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[])?;
        Ok::<_, jni::errors::Error>(())
    }).expect("simulateRegistrations should succeed");

    let entries = registry::get_entries();
    println!("Registry entries received: {}", entries.len());
    for entry in &entries {
        println!("  [{}] {}", entry.registry_type, entry.id);
    }

    assert_eq!(registry::total_count(), 12, "should have 12 total registrations (3 block + 2 item + 2 fluid + 3 entity + 2 block_entity)");
    assert!(
        entries.iter().any(|e| e.registry_type == "block" && e.id == "minecraft:stone"),
        "should contain minecraft:stone block"
    );
    assert!(
        entries.iter().any(|e| e.registry_type == "item" && e.id == "minecraft:diamond"),
        "should contain minecraft:diamond item"
    );
    assert!(
        entries.iter().any(|e| e.registry_type == "entity" && e.id == "minecraft:zombie"),
        "should contain minecraft:zombie entity"
    );
    assert!(
        entries.iter().any(|e| e.registry_type == "block_entity" && e.id == "minecraft:chest"),
        "should contain minecraft:chest block_entity"
    );
}

#[test]
fn test_event_bridge() {
    event::clear();
    init_bridge();

    let manager = shared_jvm();

    // Call Java simulateEvents() which fires events both ways
    manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("simulateEvents");
        let sig: jni::signature::RuntimeMethodSignature =
            "()V".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "simulateEvents".into(),
                    sig: "()V".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[])?;
        Ok::<_, jni::errors::Error>(())
    }).expect("simulateEvents should succeed");

    // 5 events total: 3 Java→Rust (fireToRust) + 2 Java→Rust (nativeOnEvent)
    let events = event::drain_events();
    println!("Events received: {}", events.len());
    for e in &events {
        println!("  [{}] {} ({})", e.direction, e.event_type, e.data.len());
    }
    assert_eq!(events.len(), 5, "should have 5 events (3 fireToRust + 2 nativeOnEvent)");
    assert!(
        events.iter().any(|e| e.event_type == "tick_start"),
        "should contain tick_start event"
    );
    assert!(
        events.iter().any(|e| e.event_type == "entity_damage"),
        "should contain entity_damage event"
    );
    assert!(
        events.iter().any(|e| e.event_type == "server_start"),
        "should contain server_start event (from nativeOnEvent)"
    );
    assert!(
        events.iter().any(|e| e.event_type == "player_join"),
        "should contain player_join event (from nativeOnEvent)"
    );

    // Test Rust→Java direction: call fireEvent from Rust side
    event::clear();
    manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("fireEvent");
        let sig: jni::signature::RuntimeMethodSignature =
            "(Ljava/lang/String;[B)V".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "fireEvent".into(),
                    sig: "(Ljava/lang/String;[B)V".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        let evt_type = env.new_string("rust_tick")?;
        let evt_data = env.new_byte_array(4)?;
        let buf: [i8; 4] = [0, 0, 0, 99];
        evt_data.set_region(env, 0, &buf)?;

        env.call_static_method(
            &cls_obj, &*mtd, &sig_ms,
            &[jni::objects::JValue::Object(&evt_type), jni::objects::JValue::Object(&evt_data)],
        )?;
        Ok::<_, jni::errors::Error>(())
    }).expect("fireEvent should succeed");

    let events = event::drain_events();
    println!("Events from Rust→Java: {}", events.len());
    assert_eq!(events.len(), 1, "should have 1 Rust→Java event");
    assert_eq!(events[0].event_type, "rust_tick", "event type should be rust_tick");
    assert_eq!(events[0].direction, "java_to_rust", "direction should be java_to_rust (via nativeOnEvent callback)");
}

#[test]
fn test_world_bridge() {
    world::clear();
    init_bridge();

    let manager = shared_jvm();
    world::clear();

    // Call Java simulateWorld() which pushes blocks to Rust
    manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("simulateWorld");
        let sig: jni::signature::RuntimeMethodSignature =
            "()V".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "simulateWorld".into(),
                    sig: "()V".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[])?;
        Ok::<_, jni::errors::Error>(())
    }).expect("simulateWorld should succeed");

    // Verify blocks arrived via nativeWorldSetBlock
    let count = world::block_count();
    let blocks = world::all_blocks();
    println!("World blocks received: {}", count);
    for b in &blocks {
        println!("  ({},{},{}) {}", b.position.x, b.position.y, b.position.z, b.block_id);
    }

    assert_eq!(count, 10, "should have 10 blocks (5 in chunk 0,0 + 3 in chunk 1,0 + 2 in chunk 0,1)");
    assert!(
        blocks.iter().any(|b| b.block_id == "minecraft:grass_block" && b.position.x == 0 && b.position.z == 0),
        "should contain grass_block at (0,64,0)"
    );
    assert!(
        blocks.iter().any(|b| b.block_id == "minecraft:oak_log" && b.position.x == 16),
        "should contain oak_log in chunk (1,0)"
    );
    assert!(
        blocks.iter().any(|b| b.block_id == "minecraft:water" && b.position.z == 16),
        "should contain water in chunk (0,1)"
    );

    // Test remove block
    manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/BridgeNative");
        let mtd = jni::strings::JNIString::new("nativeWorldRemoveBlock");
        let sig: jni::signature::RuntimeMethodSignature =
            "(III)V".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "nativeWorldRemoveBlock".into(),
                    sig: "(III)V".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        env.call_static_method(&cls_obj, &*mtd, &sig_ms,
            &[jni::objects::JValue::Int(0), jni::objects::JValue::Int(64), jni::objects::JValue::Int(0)])?;
        Ok::<_, jni::errors::Error>(())
    }).expect("nativeWorldRemoveBlock should succeed");

    assert_eq!(world::block_count(), 9, "should have 9 blocks after removal");
    assert!(
        world::get_block(0, 64, 0).is_none(),
        "grass_block at (0,64,0) should be removed"
    );

    // Test get_block on remaining block
    let block = world::get_block(1, 64, 0).expect("dirt should still be at (1,64,0)");
    assert_eq!(block.block_id, "minecraft:dirt", "block at (1,64,0) should be dirt");
}

#[test]
fn test_neoforge_bootstrap() {
    init_bridge();
    let manager = shared_jvm();

    // Check if NeoForge classes are loadable
    let can_load = manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("canLoadNeoForgeClasses");
        let sig: jni::signature::RuntimeMethodSignature =
            "()Z".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "canLoadNeoForgeClasses".into(),
                    sig: "()Z".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        let result = env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[])?;
        Ok::<_, jni::errors::Error>(result.z()?)
    }).expect("canLoadNeoForgeClasses should work");

    // Check if hooks were registered
    let is_hooked = manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("isNeoForgeHooked");
        let sig: jni::signature::RuntimeMethodSignature =
            "()Z".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "isNeoForgeHooked".into(),
                    sig: "()Z".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        let result = env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[])?;
        Ok::<_, jni::errors::Error>(result.z()?)
    }).expect("isNeoForgeHooked should work");

    println!("NeoForge classes loadable: {can_load}");
    println!("NeoForge hooks: {is_hooked}");

    // Either NeoForge loads and hooks, OR it falls back gracefully
    // Both are valid outcomes
    if can_load {
        println!("NeoForge is on classpath, bootstrap attempted hooking");
    }

    // The status should reflect the hook state
    let status = manager.with_env(|env| {
        let cls = jni::strings::JNIString::new("neorusty/agent/NeoRustyAgent");
        let mtd = jni::strings::JNIString::new("getStatus");
        let sig: jni::signature::RuntimeMethodSignature =
            "()Ljava/lang/String;".parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: "getStatus".into(),
                    sig: "()Ljava/lang/String;".into(),
                }
            })?;
        let sig_ms: jni::signature::MethodSignature = (&sig).into();
        let cls_obj = env.find_class(&*cls)?;
        let result = env.call_static_method(&cls_obj, &*mtd, &sig_ms, &[])?;
        let obj = result.l()?;
        let jstr = unsafe {
            jni::objects::JString::from_raw(&*env, obj.into_raw() as jni::sys::jstring)
        };
        let s = jstr.try_to_string(&*env)?;
        Ok::<_, jni::errors::Error>(s)
    }).expect("getStatus should work");

    println!("Bridge status: {status}");
    assert!(status.contains("NeoForge:"), "status should show NeoForge state");
}
