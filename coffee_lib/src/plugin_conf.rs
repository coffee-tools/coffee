//! Coffee configuration serialization file.
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq)]
pub struct Conf {
    pub plugin: Plugin,
    pub tipping: Option<Tipping>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub lang: String,
    pub deprecated: Option<()>,
    pub dependencies: Option<Vec<String>>,
    pub install: Option<String>,
    pub main: String,
    pub important: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Deprecaterd {
    pub reason: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tipping {
    pub bolt12: String,
}
