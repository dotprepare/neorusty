use crate::Plugin;
use std::any::Any;
use std::path::Path;

pub const PLUGIN_API_VERSION: u32 = 1;

pub type PluginLoadResult = Result<(Box<dyn Plugin>, PluginMetadata, Box<dyn Any + Send + Sync>), LoaderError>;

#[derive(Debug)]
pub enum LoaderError {
    LibraryLoad(String),
    ApiVersionMissing,
    ApiVersionMismatch { plugin_version: u32, server_version: u32 },
    MetadataMissing,
    EntrypointMissing,
    InvalidLoaderData,
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::LibraryLoad(e) => write!(f, "Failed to load library: {e}"),
            LoaderError::ApiVersionMissing => write!(f, "Plugin missing API version symbol"),
            LoaderError::ApiVersionMismatch { plugin_version, server_version } => {
                write!(f, "Plugin API version {plugin_version} does not match server version {server_version}")
            }
            LoaderError::MetadataMissing => write!(f, "Plugin missing metadata"),
            LoaderError::EntrypointMissing => write!(f, "Plugin missing entrypoint"),
            LoaderError::InvalidLoaderData => write!(f, "Invalid loader data"),
        }
    }
}

impl std::error::Error for LoaderError {}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub authors: &'static [&'static str],
    pub description: &'static str,
}

pub trait PluginLoader: Send + Sync {
    fn can_load(&self, path: &Path) -> bool;
    fn load(&self, path: &Path) -> PluginLoadResult;
    fn name(&self) -> &'static str;
}

pub struct NativePluginLoader;

impl PluginLoader for NativePluginLoader {
    fn can_load(&self, path: &Path) -> bool {
        let ext = path.extension().unwrap_or_default();
        if cfg!(target_os = "windows") {
            ext.eq_ignore_ascii_case("dll")
        } else if cfg!(target_os = "macos") {
            ext.eq_ignore_ascii_case("dylib")
        } else {
            ext.eq_ignore_ascii_case("so")
        }
    }

    fn load(&self, path: &Path) -> PluginLoadResult {
        let library = unsafe {
            libloading::Library::new(path)
                .map_err(|e| LoaderError::LibraryLoad(e.to_string()))?
        };

        let plugin_api_version = unsafe {
            match library.get::<*const u32>(b"NEORUSTY_API_VERSION") {
                Ok(symbol) => **symbol,
                Err(_) => return Err(LoaderError::ApiVersionMissing),
            }
        };

        if plugin_api_version != PLUGIN_API_VERSION {
            return Err(LoaderError::ApiVersionMismatch {
                plugin_version: plugin_api_version,
                server_version: PLUGIN_API_VERSION,
            });
        }

        let metadata = unsafe {
            &**library
                .get::<*const PluginMetadata>(b"NEORUSTY_METADATA")
                .map_err(|_| LoaderError::MetadataMissing)?
        };

        let plugin_factory = unsafe {
            library
                .get::<fn() -> Box<dyn Plugin>>(b"neorusty_plugin_entry")
                .map_err(|_| LoaderError::EntrypointMissing)?
        };

        Ok((
            plugin_factory(),
            metadata.clone(),
            Box::new(library) as Box<dyn Any + Send + Sync>,
        ))
    }

    fn name(&self) -> &'static str {
        "native"
    }
}

pub struct WasmPluginLoader;

impl PluginLoader for WasmPluginLoader {
    fn can_load(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext == "wasm")
            .unwrap_or(false)
    }

    fn load(&self, _path: &Path) -> PluginLoadResult {
        Err(LoaderError::LibraryLoad(
            "WASM plugin loading not yet implemented. Requires wasmtime.".to_string(),
        ))
    }

    fn name(&self) -> &'static str {
        "wasm"
    }
}
