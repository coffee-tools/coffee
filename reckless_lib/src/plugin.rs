//! Plugin module that abstract the concept of a cln plugin
//! from a plugin manager point of view.
use crate::{errors::RecklessError, plugin_conf::Conf};
use std::fmt;
use tokio::process::Command;

/// Plugin language definition
#[derive(Clone, Debug)]
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
    pub async fn default_install(&self, path: &str, name: &str) -> Result<String, RecklessError> {
        match self {
            PluginLang::Python => {
                /* 1. RUN PIP install or poetry install
                 * 2. return the path of the main file */
                let req_file = format!("{}/requirements.txt", path);
                let main_file = format!("{}/{}.py", path, name);
                match Command::new("pip")
                    .arg("install")
                    .arg("-r")
                    .arg(req_file.as_str())
                    .output()
                    .await
                {
                    Ok(_) => Ok(main_file),
                    Err(err) => {
                        return Err(RecklessError::new(
                            1,
                            &format!("problem installing python plugin {err}"),
                        ))
                    }
                }
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
#[derive(Clone)]
pub struct Plugin {
    name: String,
    path: String,
    lang: PluginLang,
    conf: Option<Conf>,
}

impl Plugin {
    /// create a new instance of the plugin.
    pub fn new(name: &str, path: &str, plugin_lang: PluginLang, config: Option<Conf>) -> Self {
        Plugin {
            name: name.to_owned(),
            path: path.to_owned(),
            lang: plugin_lang,
            conf: config,
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
                    let cmd_result = Command::new(cmd)
                        .current_dir(self.path.clone())
                        .output()
                        .await;
                    match cmd_result {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(RecklessError::new(
                                1,
                                &format!(
                                    "problem installing, error executing plugin commands : {err}"
                                ),
                            ))
                        }
                    }
                }
                format!("{}/{}", self.path, conf.plugin.main)
            } else {
                self.lang.default_install(&self.path, &self.name).await?
            }
        } else {
            self.lang.default_install(&self.path, &self.name).await?
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

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {}, path: {}", self.name, self.path)
    }
}
