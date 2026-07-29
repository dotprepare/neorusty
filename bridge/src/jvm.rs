use jni::objects::{JString, JValue};
use jni::signature::{MethodSignature, RuntimeMethodSignature};
use jni::strings::JNIString;
use jni::sys::jstring;
use jni::vm::InitArgsBuilder;
use jni::{Env, JNIVersion, JavaVM};
use std::sync::Arc;

#[derive(Debug)]
pub enum JvmError {
    Creation(String),
    Invocation(String),
    NotFound(String),
    Internal(String),
}

impl std::fmt::Display for JvmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JvmError::Creation(msg) => write!(f, "JVM creation failed: {msg}"),
            JvmError::Invocation(msg) => write!(f, "JVM invocation failed: {msg}"),
            JvmError::NotFound(msg) => write!(f, "Class/method not found: {msg}"),
            JvmError::Internal(msg) => write!(f, "JVM internal error: {msg}"),
        }
    }
}

impl std::error::Error for JvmError {}

impl From<jni::errors::Error> for JvmError {
    fn from(e: jni::errors::Error) -> Self {
        JvmError::Invocation(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct JvmConfig {
    pub class_path: Vec<String>,
    pub jvm_args: Vec<String>,
    pub min_heap_mb: u32,
    pub max_heap_mb: u32,
    pub debug: bool,
}

impl JvmConfig {
    /// Add all `.jar` files from a directory to the class path.
    pub fn add_lib_dir(&mut self, dir: &str) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jar") {
                if let Some(s) = path.to_str() {
                    if !self.class_path.iter().any(|cp| cp == s) {
                        self.class_path.push(s.to_string());
                    }
                }
            }
        }
    }
}

impl Default for JvmConfig {
    fn default() -> Self {
        Self {
            class_path: Vec::new(),
            jvm_args: Vec::new(),
            min_heap_mb: 128,
            max_heap_mb: 2048,
            debug: false,
        }
    }
}

pub struct JvmManager {
    pub jvm: Arc<JavaVM>,
}

impl JvmManager {
    pub fn start() -> Result<Self, JvmError> {
        Self::start_with_config(JvmConfig::default())
    }

    pub fn start_with_config(config: JvmConfig) -> Result<Self, JvmError> {
        let mut builder = InitArgsBuilder::new();
        builder = builder.version(JNIVersion::V21);

        builder = builder.option(format!("-Xms{}m", config.min_heap_mb));
        builder = builder.option(format!("-Xmx{}m", config.max_heap_mb));

        for arg in &config.jvm_args {
            builder = builder.option(arg);
        }

        if config.debug {
            builder = builder.option("-verbose:jni");
        }

        let mut classpath = config.class_path.join(":");
        if classpath.is_empty() {
            classpath = ".".to_string();
        }
        builder = builder.option(format!("-Djava.class.path={classpath}"));

        let init_args = builder
            .build()
            .map_err(|e| JvmError::Creation(e.to_string()))?;

        let jvm = JavaVM::new(init_args)
            .map_err(|e| JvmError::Creation(e.to_string()))?;

        Ok(Self {
            jvm: Arc::new(jvm),
        })
    }

    pub fn with_env<F, T>(&self, f: F) -> Result<T, JvmError>
    where
        F: FnOnce(&mut Env) -> Result<T, jni::errors::Error>,
    {
        self.jvm
            .attach_current_thread(|env| f(env))
            .map_err(|e| JvmError::Invocation(e.to_string()))
    }

    pub fn get_version(&self) -> Result<String, JvmError> {
        self.with_env(|env| {
            let class = JNIString::new("java/lang/System");
            let method = JNIString::new("getProperty");
            let sig: RuntimeMethodSignature =
                "(Ljava/lang/String;)Ljava/lang/String;".parse().map_err(|_| {
                    jni::errors::Error::MethodNotFound {
                        name: "getProperty".into(),
                        sig: "(Ljava/lang/String;)Ljava/lang/String;".into(),
                    }
                })?;
            let sig_ms: MethodSignature = (&sig).into();

            let sys_cls = env.find_class(&*class)?;
            let jstr = env.new_string("java.version")?;

            let result = env.call_static_method(
                &sys_cls, &*method, &sig_ms, &[JValue::Object(&jstr)],
            )?;
            let obj = result.l()?;
            let jstr_result = unsafe { JString::from_raw(&*env, obj.into_raw() as jstring) };
            let s = jstr_result.try_to_string(&*env)?;
            Ok(s)
        })
    }

    pub fn call_static_int(
        &self,
        class: &str,
        method: &str,
        sig: &str,
        args: &[JValue],
    ) -> Result<i32, JvmError> {
        self.with_env(|env| {
            let class_str = JNIString::new(class);
            let method_str = JNIString::new(method);
            let sig_parsed: RuntimeMethodSignature = sig.parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: method.into(),
                    sig: sig.into(),
                }
            })?;
            let sig_ms: MethodSignature = (&sig_parsed).into();
            let cls = env.find_class(&*class_str)?;
            let result = env.call_static_method(&cls, &*method_str, &sig_ms, args)?;
            Ok(result.i()?)
        })
    }

    pub fn call_static_bool(
        &self,
        class: &str,
        method: &str,
        sig: &str,
        args: &[JValue],
    ) -> Result<bool, JvmError> {
        self.with_env(|env| {
            let class_str = JNIString::new(class);
            let method_str = JNIString::new(method);
            let sig_parsed: RuntimeMethodSignature = sig.parse().map_err(|_| {
                jni::errors::Error::MethodNotFound {
                    name: method.into(),
                    sig: sig.into(),
                }
            })?;
            let sig_ms: MethodSignature = (&sig_parsed).into();
            let cls = env.find_class(&*class_str)?;
            let result = env.call_static_method(&cls, &*method_str, &sig_ms, args)?;
            Ok(result.z()?)
        })
    }

}

impl Drop for JvmManager {
    fn drop(&mut self) {
        if let Some(jvm) = Arc::get_mut(&mut self.jvm) {
            let _ = unsafe { jvm.destroy() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn shared_jvm() -> &'static JvmManager {
        static JVM: OnceLock<JvmManager> = OnceLock::new();
        JVM.get_or_init(|| JvmManager::start().expect("JVM should start"))
    }

    #[test]
    fn test_start_jvm() {
        let manager = shared_jvm();
        let version = manager.get_version().expect("get version");
        assert!(!version.is_empty(), "Java version should not be empty");
        println!("Java version: {version}");
    }

    #[test]
    fn test_call_static_method() {
        let manager = shared_jvm();
        let result = manager.call_static_int(
            "java/lang/Math",
            "max",
            "(II)I",
            &[JValue::from(42), JValue::from(7)],
        );
        assert_eq!(result.unwrap(), 42);
    }
}
