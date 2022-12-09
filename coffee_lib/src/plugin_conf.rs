//! Coffee configuration serialization file.
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]

pub struct Conf {
    pub plugin: Plugin,
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_remote() {}
}
