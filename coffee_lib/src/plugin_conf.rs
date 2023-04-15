//! Coffee configuration serialization file.
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Conf {
    pub plugin: Option<Plugin>,
    pub bin: Option<Plugin>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub lang: Option<String>,
    /// Is the plugin a binary?
    pub binary: Option<bool>,
    pub deprecated: Option<Deprecaterd>,
    pub dependencies: Option<Vec<String>>,
    pub install: Option<String>,
    pub main: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Deprecaterd {
    pub reason: String,
}
