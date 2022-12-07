use std::fmt::format;

use async_trait::async_trait;
use git2;
use glob::glob;
use log::debug;
use reckless_lib::errors::RecklessError;
use reckless_lib::plugin::Plugin;
use reckless_lib::plugin::PluginLang;
use reckless_lib::plugin_conf::Conf;
use reckless_lib::repository::Repository;
use reckless_lib::url::URL;
use reckless_lib::utils::clone_recursive_fix;
use reckless_lib::utils::get_plugin_info_from_path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct Github {
    /// the url of the repository to be able
    /// to get all the plugin information.
    url: URL,
    /// the name of the repository that can be used
    /// by reckless as repository key.
    name: String,
    /// all the plugin that are listed inside the
    /// repository
    plugins: Vec<Plugin>,
}

impl Github {
    /// Create a new instance of the Repository
    /// with a name and a url
    pub fn new(name: &str, url: &URL) -> Self {
        debug!("ADDING REPOSITORY: {} {}", name, url.url_string);
        Github {
            name: name.to_owned(),
            url: url.clone(),
            plugins: vec![],
        }
    }

    /// Index the repository to store information
    /// related to the plugins
    pub async fn index_repository(&mut self) -> Result<(), RecklessError> {
        let repo_path = &self.url.path_string;
        let pattern = format!("{}/[!.]*/*", &repo_path);

        // FIXME rewrite it in a way that is more clear that
        // we are walking all the plugins.
        // for plugin_dir in repo_pat:
        //    let conf = None
        //    let plugin = None
        //    for file in plugin_dir:
        //
        //     let lang = match file {
        //          ...
        //      }
        //      plugun = {conf, lang ...}
        //      self.plugins(plugin)
        for plugin in glob(&pattern).unwrap() {
            match plugin {
                Ok(path) => {
                    let (path_to_plugin, plugin_name) =
                        get_plugin_info_from_path(path.clone()).unwrap();

                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    match file_name {
                        "requirements.txt" => {
                            self.plugins.push(Plugin::new(
                                &plugin_name,
                                &path_to_plugin,
                                PluginLang::Python,
                            ));
                        }
                        "go.mod" => {
                            self.plugins.push(Plugin::new(
                                &plugin_name,
                                &path_to_plugin,
                                PluginLang::Go,
                            ));
                        }
                        "cargo.toml" => {
                            self.plugins.push(Plugin::new(
                                &plugin_name,
                                &path_to_plugin,
                                PluginLang::Rust,
                            ));
                        }
                        "pubspec.yaml" => {
                            self.plugins.push(Plugin::new(
                                &plugin_name,
                                &path_to_plugin,
                                PluginLang::Dart,
                            ));
                        }
                        "package.json" => {
                            self.plugins.push(Plugin::new(
                                &plugin_name,
                                &path_to_plugin,
                                PluginLang::JavaScript,
                            ));
                        }
                        "tsconfig.json" => {
                            // FIXME: avoid to unwrap here
                            self.plugins.push(Plugin::new(
                                plugin_name.as_str(),
                                path_to_plugin.as_str(),
                                PluginLang::TypeScript,
                            ));
                        }
                        "reckless.yml" | "reckless.yaml" => {
                            let conf_path = format!("{}/{}", path_to_plugin, file_name);
                            let mut conf_str = String::new();
                            File::open(conf_path.as_str())
                                .await?
                                .read_to_string(&mut conf_str)
                                .await?;
                            let _: Conf = serde_yaml::from_str(&conf_str).unwrap();
                            // FIXME: store the conf inside the plugin
                        }
                        _ => continue,
                    }
                }
                Err(err) => return Err(RecklessError::new(1, err.to_string().as_str())),
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Repository for Github {
    /// Init the repository where it is required to index
    /// all the plugin contained, and store somewhere the index.
    ///
    /// Where to store the index is an implementation
    /// details.
    async fn init(&mut self) -> Result<(), RecklessError> {
        debug!(
            "INITIALIZING REPOSITORY: {} {} > {}",
            self.name, &self.url.url_string, &self.url.path_string,
        );
        let res = git2::Repository::clone(&self.url.url_string, &self.url.path_string);
        match res {
            Ok(repo) => {
                let clone = clone_recursive_fix(repo, &self.url);
                self.index_repository().await?;
                clone
            }
            Err(err) => Err(RecklessError::new(1, err.message())),
        }
    }

    /// list of the plugin installed inside the repository.
    ///
    /// M.B: in the future we want also list all the plugin installed
    /// inside the repository.
    async fn list(&self) -> Result<Vec<Plugin>, RecklessError> {
        Ok(self.plugins.clone())
    }
}
