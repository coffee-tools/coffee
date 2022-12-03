//! Plugin module that abstract the concept of a cln plugin
//! from a plugin manager point of view.
use std::fmt;

use crate::{errors::RecklessError, plugin_conf::Conf};

/// Plugin language definition
pub enum PluginLang {
    Python,
    Go,
    Rust,
    Dart,
    JavaScript,
    TypeScript,
    Unknown,
}

impl PluginLang {
    pub fn default_install(&self) -> Result<String, RecklessError> {
        match self {
            PluginLang::Python => {
                /* 1. RUN PIP install or poetry install
                 * 2. return the path of the main file */
                todo!()
            }
            PluginLang::Go => {
                /* better instructions needed here */
                todo!()
            }
            PluginLang::Rust => {
                /* 1. run cargo build in release mode
                 * 2. return the binary path */
                todo!()
            }
            PluginLang::Dart => {
                /* 1. run dart compile exe and
                 * 2. return the binary path */
                todo!()
            }
            PluginLang::JavaScript => {
                /* better instructions needed here */
                todo!()
            }
            PluginLang::TypeScript => {
                /* 1. From https://github.com/runcitadel/core-ln.ts
                deno run --allow-env --allow-read --allow-write src/generate.ts
                 * 2. run the ts file */
                todo!()
            }
            PluginLang::Unknown => {
                /* 1. emit an error message  */
                let err = RecklessError::new(
                    2,
                    "unknown default install procedure, the language in undefined",
                );
                Err(err)
            }
        }
    }
}

/// Plugin struct definition
pub struct Plugin {
    name: String,
    path: String,
    lang: PluginLang,
    conf: Option<Conf>,
}

impl Plugin {
    /// create a new instance of the plugin.
    pub fn new(name: &str, path: &str, plugin_lang: PluginLang) -> Self {
        Plugin {
            name: name.to_owned(),
            path: path.to_owned(),
            lang: plugin_lang,
            conf: None,
        }
    }

    /// configure the plugin in order to work with cln.
    ///
    /// In case of success return the path of the executable.
    pub async fn configure(&mut self) -> Result<String, RecklessError> {
        let exec_path = if let Some(conf) = &self.conf {
            if let Some(script) = &conf.plugin.install {
                let cmds = script.split("\n"); // Check if the script contains `\`
                for cmd in cmds {
                    // FIXME: run command
                }
                format!("{}/{}", self.path, conf.plugin.main)
            } else {
                self.lang.default_install()?
            }
        } else {
            self.lang.default_install()?
        };
        Ok(exec_path)
    }

    /// upgrade the plugin to a new version.
    pub async fn upgrade(&mut self) -> Result<(), RecklessError> {
        todo!("not implemented yet")
    }

    /// remove the plugin and clean up all the data.
    async fn remove(&mut self) -> Result<(), RecklessError> {
        todo!("not implemented yet")
    }
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {}, path: {}", self.name, self.path)
    }
}
