//! Coffee configuration serialization file.
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Conf {
    pub plugin: Plugin,
    pub tipping: Option<Tipping>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub lang: String,
    pub deprecated: Option<()>,
    pub dependencies: Option<Vec<String>>,
    pub install: Option<String>,
    pub main: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Deprecaterd {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tipping {
    pub bolt12: String,
}
