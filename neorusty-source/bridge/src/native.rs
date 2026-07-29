use jni::objects::{JByteArray, JClass, JString};
use jni::Env;
use jni::strings::{JNIStr, JNIString};
use jni::sys::jlong;
use jni::EnvUnowned;
use jni::JavaVM;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64};

use crate::event;
use crate::registry::{self, RegistryEntry};
use crate::world;

pub static BRIDGE_INITIALIZED: AtomicBool = AtomicBool::new(false);
pub static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// Register all native callback implementations with the JVM.
pub fn register_natives(jvm: &JavaVM) -> Result<(), String> {
    jvm.attach_current_thread(|env| {
        let class_str = JNIString::new("neorusty/agent/BridgeNative");
        env.find_class(&*class_str).map_err(|_| {
            jni::errors::Error::ClassNotFound {
                name: "neorusty/agent/BridgeNative".to_string(),
            }
        })?;

        let methods = native_methods();
        unsafe {
            env.register_native_methods(&*class_str, &methods)?;
        }

        Ok::<_, jni::errors::Error>(())
    })
    .map_err(|e| format!("register_natives failed: {e}"))?;

    Ok(())
}

fn native_methods() -> Vec<jni::NativeMethod<'static>> {
    // Leak the strings to get 'static lifetimes for the NativeMethod descriptors.
    // These are only created once and live for the program's duration.
    fn leak(s: JNIString) -> &'static JNIStr {
        Box::leak(Box::new(s))
    }

    vec![
        nm(leak(JNIString::new("nativeOnInit")), leak(JNIString::new("(Ljava/lang/String;Ljava/lang/String;)Z")), native_on_init as *mut c_void),
        nm(leak(JNIString::new("nativeOnEvent")), leak(JNIString::new("(Ljava/lang/String;[B)V")), native_on_event as *mut c_void),
        nm(leak(JNIString::new("nativeOnTick")), leak(JNIString::new("(J)V")), native_on_tick as *mut c_void),
        nm(leak(JNIString::new("nativeRegisterBlock")), leak(JNIString::new("(Ljava/lang/String;[B)V")), native_register_block as *mut c_void),
        nm(leak(JNIString::new("nativeRegisterItem")), leak(JNIString::new("(Ljava/lang/String;[B)V")), native_register_item as *mut c_void),
        nm(leak(JNIString::new("nativeRegisterFluid")), leak(JNIString::new("(Ljava/lang/String;[B)V")), native_register_fluid as *mut c_void),
        nm(leak(JNIString::new("nativeRegisterEntity")), leak(JNIString::new("(Ljava/lang/String;[B)V")), native_register_entity as *mut c_void),
        nm(leak(JNIString::new("nativeRegisterBlockEntity")), leak(JNIString::new("(Ljava/lang/String;[B)V")), native_register_block_entity as *mut c_void),
        nm(leak(JNIString::new("nativeFireEvent")), leak(JNIString::new("(Ljava/lang/String;[B)V")), native_fire_event as *mut c_void),
        nm(leak(JNIString::new("nativeLog")), leak(JNIString::new("(ILjava/lang/String;)V")), native_log as *mut c_void),
        nm(leak(JNIString::new("nativeWorldSetBlock")), leak(JNIString::new("(IIILjava/lang/String;[B)V")), native_world_set_block as *mut c_void),
        nm(leak(JNIString::new("nativeWorldRemoveBlock")), leak(JNIString::new("(III)V")), native_world_remove_block as *mut c_void),
        nm(leak(JNIString::new("nativeWorldGetBlockCount")), leak(JNIString::new("()I")), native_world_get_block_count as *mut c_void),
    ]
}

fn nm(name: &'static JNIStr, sig: &'static JNIStr, fn_ptr: *mut c_void) -> jni::NativeMethod<'static> {
    unsafe { jni::NativeMethod::from_raw_parts(name, sig, fn_ptr) }
}

// --- Native method implementations ---

unsafe extern "system" fn native_on_init(
    _env: EnvUnowned,
    _class: JClass,
    _neo_forge_dir: JString,
    _mods_dir: JString,
) -> bool {
    BRIDGE_INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst);
    true
}

unsafe extern "system" fn native_on_event<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    event_type: JString<'local>,
    event_data: JByteArray<'local>,
) {
    let _ = env.with_env(|env| -> std::result::Result<(), jni::errors::Error> {
        let type_str = event_type.try_to_string(&*env)?;
        let bytes = env.convert_byte_array(&event_data)?;
        let u8vec = bytes.iter().map(|&b| b as u8).collect();
        event::push_event(&type_str, u8vec, "java_to_rust");
        Ok(())
    });
}

unsafe extern "system" fn native_fire_event<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    event_type: JString<'local>,
    event_data: JByteArray<'local>,
) {
    let _ = env.with_env(|env| -> std::result::Result<(), jni::errors::Error> {
        let type_str = event_type.try_to_string(&*env)?;
        let bytes = env.convert_byte_array(&event_data)?;
        let u8vec = bytes.iter().map(|&b| b as u8).collect();
        event::push_event(&type_str, u8vec, "java_to_rust_fire");
        Ok(())
    });
}

unsafe extern "system" fn native_on_tick(
    _env: EnvUnowned,
    _class: JClass,
    tick: jlong,
) {
    TICK_COUNT.store(tick as u64, std::sync::atomic::Ordering::SeqCst);
}

unsafe extern "system" fn native_log(
    _env: EnvUnowned,
    _class: JClass,
    level: jni::sys::jint,
    message: JString,
) {
    let msg = message.to_string();
    match level {
        0 => eprintln!("[NeoRusty/ERROR] {msg}"),
        1 => println!("[NeoRusty] {msg}"),
        2 => println!("[NeoRusty/DEBUG] {msg}"),
        _ => println!("[NeoRusty/LOG] {msg}"),
    }
}

// --- Registry native methods (Java→Rust: mod registrations) ---

fn register_through_env(env: &mut Env, id: &JString, data: &JByteArray, reg_type: &str) {
    let id_str = match id.try_to_string(&*env) {
        Ok(s) => s,
        Err(_) => return,
    };
    let bytes = match env.convert_byte_array(data) {
        Ok(b) => b.iter().map(|&b| b as u8).collect::<Vec<_>>(),
        Err(_) => return,
    };
    registry::push_entry(RegistryEntry::new(reg_type, &id_str, bytes));
}

macro_rules! make_register_fn {
    ($name:ident, $type:literal) => {
        unsafe extern "system" fn $name<'local>(
            mut env: EnvUnowned<'local>,
            _class: JClass<'local>,
            id: JString<'local>,
            data: JByteArray<'local>,
        ) {
            let _ = env.with_env(|env| -> std::result::Result<(), jni::errors::Error> {
                register_through_env(env, &id, &data, $type);
                Ok(())
            });
        }
    };
}

make_register_fn!(native_register_block, "block");
make_register_fn!(native_register_item, "item");
make_register_fn!(native_register_fluid, "fluid");
make_register_fn!(native_register_entity, "entity");
make_register_fn!(native_register_block_entity, "block_entity");

// --- World native methods ---

unsafe extern "system" fn native_world_set_block<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    x: jni::sys::jint,
    y: jni::sys::jint,
    z: jni::sys::jint,
    block_id: JString<'local>,
    data: JByteArray<'local>,
) {
    let _ = env.with_env(|env| -> std::result::Result<(), jni::errors::Error> {
        let id_str = block_id.try_to_string(&*env)?;
        let bytes = env.convert_byte_array(&data)?;
        let u8vec = bytes.iter().map(|&b| b as u8).collect();
        world::set_block(x, y, z, &id_str, u8vec);
        Ok(())
    });
}

unsafe extern "system" fn native_world_remove_block<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    x: jni::sys::jint,
    y: jni::sys::jint,
    z: jni::sys::jint,
) {
    let _ = env.with_env(|_env| -> std::result::Result<(), jni::errors::Error> {
        world::remove_block(x, y, z);
        Ok(())
    });
}

unsafe extern "system" fn native_world_get_block_count(
    _env: EnvUnowned,
    _class: JClass,
) -> jni::sys::jint {
    world::block_count() as jni::sys::jint
}
