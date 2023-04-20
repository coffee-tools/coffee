use std::any::Any;

use async_trait::async_trait;
use git2;
use log::debug;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use walkdir::DirEntry;
use walkdir::WalkDir;

use coffee_lib::errors::CoffeeError;
use coffee_lib::macros::error;
use coffee_lib::plugin::Plugin;
use coffee_lib::plugin::PluginLang;
use coffee_lib::plugin_conf::Conf;
use coffee_lib::repository::Repository;
use coffee_lib::url::URL;
use coffee_lib::utils::get_plugin_info_from_path;
use coffee_storage::model::repository::Kind;
use coffee_storage::model::repository::Repository as StorageRepository;

use crate::utils::clone_recursive_fix;

pub struct Github {
    /// the url of the repository to be able
    /// to get all the plugin information.
    url: URL,
    /// the name of the repository that can be used
    /// by coffee as repository key.
    name: String,
    /// all the plugin that are listed inside the
    /// repository
    plugins: Vec<Plugin>,
}

// FIXME: move this inside a utils dir craters
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

impl Github {
    /// Create a new instance of the Repository
    /// with a name and a url
    pub fn new(name: &str, url: &URL) -> Self {
        debug!("creating repository: {} {}", name, url.url_string);
        Github {
            name: name.to_owned(),
            url: url.clone(),
            plugins: vec![],
        }
    }

    /// Index the repository to store information
    /// related to the plugins
    pub async fn index_repository(&mut self) -> Result<(), CoffeeError> {
        let repo_path = &self.url.path_string;
        let target_dirs = WalkDir::new(repo_path)
            .max_depth(1)
            .into_iter()
            .filter_entry(|dir_entry| !is_hidden(dir_entry));

        for plugin_dir in target_dirs {
            match plugin_dir {
                Ok(plugin_path) => {
                    let root_path = plugin_path
                        .path()
                        .as_os_str()
                        .to_os_string()
                        .to_string_lossy()
                        .to_string();
                    let mut path_to_plugin = None;
                    let mut plugin_name = None;
                    let mut plugin_lang = PluginLang::Unknown;

                    // check if the plugin has the custom configuration to read.
                    let mut conf = None;
                    for file in ["coffee.yaml", "coffee.yml"] {
                        let conf_path = format!("{}/{}", root_path, file);
                        if let Ok(mut conf_file) = File::open(conf_path).await {
                            let mut conf_str = String::new();
                            conf_file.read_to_string(&mut conf_str).await?;
                            debug!("found plugin configuration: {}", conf_str);

                            let conf_file = serde_yaml::from_str::<Conf>(&conf_str)
                                .map_err(|err| error!("Coffee manifest malformed: {err}"))?;
                            plugin_name = Some(conf_file.plugin.name.to_string());
                            path_to_plugin =
                                Some(format!("{}/{}", root_path, &conf_file.plugin.main));
                            let conf_lang = (&conf_file.plugin.lang).to_owned();
                            match conf_lang.as_str() {
                                "pypip" => plugin_lang = PluginLang::PyPip,
                                "pypoetry" => plugin_lang = PluginLang::PyPoetry,
                                "go" => plugin_lang = PluginLang::Go,
                                "rs" | "rust" => plugin_lang = PluginLang::Rust,
                                "dart" => plugin_lang = PluginLang::Dart,
                                "js" => plugin_lang = PluginLang::JavaScript,
                                "ts" => plugin_lang = PluginLang::TypeScript,
                                "java" | "kotlin" | "scala" => plugin_lang = PluginLang::JVM,
                                _ => {
                                    return Err(error!("language {conf_lang} not supported"));
                                }
                            };

                            conf = Some(conf_file);
                            break;
                        }
                    }

                    // check if there was a coffee configuration file
                    if conf == None {
                        debug!("conf file not found, so we try to guess the language");
                        // try to understand the language from the file
                        let files = WalkDir::new(plugin_path.path()).max_depth(1);
                        for file in files {
                            let file_dir = file.unwrap().clone();
                            let (tmp_root_path, tmp_plugin_name) =
                                get_plugin_info_from_path(file_dir.path()).unwrap();
                            plugin_name = Some(tmp_plugin_name.to_string());
                            debug!("looking for {tmp_plugin_name} in {tmp_root_path}");
                            let file_name = file_dir.file_name().to_str().unwrap();
                            match file_name {
                                "requirements.txt" => {
                                    plugin_lang = PluginLang::PyPip;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.py"))
                                }
                                "pyproject.toml" => {
                                    plugin_lang = PluginLang::PyPoetry;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.py"))
                                }
                                "go.mod" => {
                                    plugin_lang = PluginLang::Go;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.go"))
                                }
                                "cargo.toml" => {
                                    plugin_lang = PluginLang::Rust;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.rs"))
                                }
                                "pubspec.yaml" => {
                                    plugin_lang = PluginLang::Dart;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.dart"))
                                }
                                "package.json" => {
                                    plugin_lang = PluginLang::JavaScript;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.js"))
                                }
                                "tsconfig.json" => {
                                    plugin_lang = PluginLang::TypeScript;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.ts"))
                                }
                                _ => {
                                    plugin_lang = PluginLang::Unknown;
                                    path_to_plugin =
                                        Some(format!("{tmp_root_path}/{tmp_plugin_name}.ts"))
                                }
                            }
                            if plugin_lang != PluginLang::Unknown {
                                break;
                            }
                        }
                        debug!("possible plugin language: {:?}", plugin_lang);
                    }

                    // The language is already contained inside the configuration file.
                    let plugin = Plugin::new(
                        plugin_name.unwrap().as_str(),
                        &root_path,
                        path_to_plugin.unwrap().as_str(),
                        plugin_lang,
                        conf.clone(),
                    );

                    debug!("new plugin: {:?}", plugin);
                    self.plugins.push(plugin);
                }
                Err(err) => return Err(error!("{}", err)),
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
    async fn init(&mut self) -> Result<(), CoffeeError> {
        debug!(
            "initializing repository: {} {} > {}",
            self.name, &self.url.url_string, &self.url.path_string,
        );
        let res = git2::Repository::clone(&self.url.url_string, &self.url.path_string);
        match res {
            Ok(repo) => {
                let clone = clone_recursive_fix(repo, &self.url).await;
                self.index_repository().await?;
                clone
            }
            Err(err) => Err(CoffeeError::new(1, err.message())),
        }
    }

    /// list of the plugin installed inside the repository.
    async fn list(&self) -> Result<Vec<Plugin>, CoffeeError> {
        Ok(self.plugins.clone())
    }

    /// name of the repository.
    fn name(&self) -> String {
        self.name.clone()
    }

    /// url of the repository.
    fn url(&self) -> URL {
        self.url.clone()
    }

    /// search inside the repository a plugin by name.
    fn get_plugin_by_name(&self, name: &str) -> Option<Plugin> {
        for plugin in &self.plugins {
            if plugin.name() == name {
                return Some(plugin.to_owned());
            }
        }
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<StorageRepository> for Github {
    fn from(value: StorageRepository) -> Self {
        Github {
            url: value.url,
            name: value.name,
            plugins: value.plugins,
        }
    }
}

impl From<&StorageRepository> for Github {
    fn from(value: &StorageRepository) -> Self {
        Github {
            url: value.url.to_owned(),
            name: value.name.to_owned(),
            plugins: value.plugins.to_owned(),
        }
    }
}

impl From<Github> for StorageRepository {
    fn from(value: Github) -> Self {
        StorageRepository {
            kind: Kind::Git,
            name: value.name,
            url: value.url,
            plugins: value.plugins,
        }
    }
}

impl From<&Github> for StorageRepository {
    fn from(value: &Github) -> Self {
        StorageRepository {
            kind: Kind::Git,
            name: value.name.to_owned(),
            url: value.url.to_owned(),
            plugins: value.plugins.to_owned(),
        }
    }
}
