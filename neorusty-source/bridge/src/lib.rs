pub mod event;
pub mod jvm;
pub mod native;
pub mod registry;
pub mod worker;
pub mod world;

pub use jvm::*;
pub use native::*;

/// Full bridge initialization: start JVM, register Java agent, wire native callbacks.
pub fn init_bridge(
    agent_jar: &str,
) -> Result<jvm::JvmManager, Box<dyn std::error::Error>> {
    let mut config = jvm::JvmConfig::default();
    config.class_path.push(agent_jar.to_string());
    config.debug = true;

    let manager = jvm::JvmManager::start_with_config(config)?;

    native::register_natives(&manager.jvm)?;

    // Call NeoRustyAgent.init() from Rust inside a JNI env context
    manager.with_env(|env| {
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
            &cls_obj,
            &*mtd,
            &sig_ms,
            &[jni::objects::JValue::Object(&dir), jni::objects::JValue::Object(&mods)],
        )?;

        let ok = result.z()?;
        if !ok {
            return Err(jni::errors::Error::JavaException);
        }
        Ok(())
    })?;

    Ok(manager)
}
