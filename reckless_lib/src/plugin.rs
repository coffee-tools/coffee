//! Plugin module that abstract the concept of a cln plugin
//! from a plugin manager point of view.
use crate::errors::RecklessError;

/// Plugin language definition
pub enum PluginLang {
    Python,
    Rust,
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
            PluginLang::Rust => {
                /* 1. run cargo build in release mode
                 * 2. return the binary path */
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
    /// plugin language
    lang: PluginLang,
    // FIXME: the plugin should contains also
    // a custom install script stored in some way
    custom_install: Option<()>,
}

impl Plugin {
    /// create a new instance of the plugin.
    pub fn new() -> Self {
        Plugin {
            lang: PluginLang::Unknown,
            custom_install: None,
        }
    }

    /// configure the plugin in order to work with cln.
    ///
    /// In case of success return the path of the executable.
    pub async fn configure(&mut self) -> Result<String, RecklessError> {
        let exec_path = if let Some(_) = self.custom_install {
            todo!()
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
