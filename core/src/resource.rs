use std::fmt;
use std::sync::LazyLock;

static SEPARATOR: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new("^([a-z0-9_.-]+):([a-z0-9_./-]+)$").unwrap());

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceLocation {
    namespace: String,
    path: String,
}

impl ResourceLocation {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        if let Some(caps) = SEPARATOR.captures(s) {
            Ok(Self {
                namespace: caps[1].to_string(),
                path: caps[2].to_string(),
            })
        } else if !s.contains(':') {
            Ok(Self {
                namespace: "minecraft".to_string(),
                path: s.to_string(),
            })
        } else {
            Err(format!("Invalid ResourceLocation: {s}"))
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn with_path(&self, path: impl Into<String>) -> Self {
        Self {
            namespace: self.namespace.clone(),
            path: path.into(),
        }
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

fn default_namespace(s: &str) -> ResourceLocation {
    ResourceLocation {
        namespace: "minecraft".to_string(),
        path: s.to_string(),
    }
}

impl From<&str> for ResourceLocation {
    fn from(s: &str) -> Self {
        Self::parse(s).unwrap_or_else(|_| default_namespace(s))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceKey<T> {
    registry: ResourceLocation,
    location: ResourceLocation,
    _marker: std::marker::PhantomData<T>,
}

impl<T> ResourceKey<T> {
    pub fn new(registry: ResourceLocation, location: ResourceLocation) -> Self {
        Self {
            registry,
            location,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn registry(&self) -> &ResourceLocation {
        &self.registry
    }

    pub fn location(&self) -> &ResourceLocation {
        &self.location
    }
}

impl<T: 'static> std::fmt::Display for ResourceKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.registry, self.location)
    }
}
