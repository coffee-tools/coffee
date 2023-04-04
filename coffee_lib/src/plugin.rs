//! Plugin module that abstract the concept of a cln plugin
//! from a plugin manager point of view.
use std::fmt;

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::errors::CoffeeError;
use crate::macros::error;
use crate::plugin_conf::Conf;

/// Plugin language definition
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PluginLang {
    PyPip,
    PyPoetry,
    Go,
    Rust,
    Dart,
    JVM,
    JavaScript,
    TypeScript,
    Unknown,
}

impl PluginLang {
    pub async fn default_install(
        &self,
        path: &str,
        name: &str,
        verbose: bool,
    ) -> Result<String, CoffeeError> {
        match self {
            PluginLang::PyPip => {
                /* 1. RUN PIP install or poetry install
                 * 2. return the path of the main file */
                let req_file = format!("{path}/requirements.txt");
                let main_file = format!("{path}/{name}.py");
                let mut cmd = Command::new("pip");
                cmd.arg("install").arg("-r").arg(&req_file.clone());
                if verbose {
                    let _ = cmd
                        .spawn()
                        .expect("Unable to run the command")
                        .wait()
                        .await?;
                } else {
                    let _ = cmd.output().await?;
                }
                Ok(main_file)
            }
            PluginLang::PyPoetry => {
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
            PluginLang::JVM => todo!(),
            PluginLang::Unknown => {
                /* 1. emit an error message  */
                Err(error!(
                    "unknown default install procedure, the language in undefined"
                ))
            }
        }
    }
}

/// Plugin struct definition
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Plugin {
    name: String,
    /// root path of the plugin
    root_path: String,
    /// path of the main file
    pub path: String,
    lang: PluginLang,
    conf: Option<Conf>,
}

impl Plugin {
    /// create a new instance of the plugin.
    pub fn new(
        name: &str,
        root_path: &str,
        path: &str,
        plugin_lang: PluginLang,
        config: Option<Conf>,
    ) -> Self {
        Plugin {
            name: name.to_owned(),
            root_path: root_path.to_owned(),
            path: path.to_owned(),
            lang: plugin_lang,
            conf: config,
        }
    }

    /// configure the plugin in order to work with cln.
    ///
    /// In case of success return the path of the executable.
    pub async fn configure(&mut self, verbose: bool) -> Result<String, CoffeeError> {
        let exec_path = if let Some(conf) = &self.conf {
            if let Some(script) = &conf.plugin.install {
                let script = script.trim();
                let cmds = script.split("\n"); // Check if the script contains `\`
                debug!("cmds: {:#?}", cmds);
                for cmd in cmds {
                    debug!("cmd {:#?}", cmd);
                    let cmd_tok: Vec<&str> = cmd.split(" ").collect();
                    let command = cmd_tok.first().unwrap().to_string();
                    let mut cmd = Command::new(command);
                    cmd.args(&cmd_tok[1..cmd_tok.len()]);
                    cmd.current_dir(self.root_path.clone());
                    if verbose {
                        let _ = cmd
                            .spawn()
                            .expect("Unable to run the command")
                            .wait()
                            .await?;
                    } else {
                        let _ = cmd.output().await?;
                    }
                }
                format!("{}/{}", self.path, conf.plugin.main)
            } else {
                self.lang
                    .default_install(&self.path, &self.name, verbose)
                    .await?
            }
        } else {
            self.lang
                .default_install(&self.path, &self.name, verbose)
                .await?
        };
        Ok(exec_path)
    }

    /// upgrade the plugin to a new version.
    pub async fn upgrade(&mut self) -> Result<(), CoffeeError> {
        todo!("not implemented yet")
    }

    /// remove the plugin and clean up all the data.
    async fn remove(&mut self) -> Result<(), CoffeeError> {
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
