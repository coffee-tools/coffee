use std::any::Any;

use async_trait::async_trait;
use coffee_lib::types::CoffeeUpgrade;
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
use crate::utils::fast_forward;

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
    /// the name of the branch to be able to
    /// pull the changes from the correct branch
    branch: String,
}

// FIXME: move this inside a utils dir craters
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
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
            branch: "".to_owned(),
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
                    let mut exec_path = None;
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
                            let conf_lang = conf_file.plugin.lang.to_owned();
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

                            exec_path = Some(format!("{root_path}/{}", conf_file.plugin.main));
                            conf = Some(conf_file);
                            break;
                        }
                    }

                    // check if there was a coffee configuration file
                    if conf.is_none() {
                        debug!("conf file not found, so we try to guess the language");
                        // try to understand the language from the file
                        let files = WalkDir::new(plugin_path.path()).max_depth(1);
                        for file in files {
                            let file_dir = file.unwrap().clone();
                            let (derived_root_path, derived_name) =
                                get_plugin_info_from_path(file_dir.path())?;

                            plugin_name = Some(derived_name.to_string());
                            debug!("looking for {derived_name} in {derived_root_path}");
                            let file_name = file_dir.file_name().to_str().unwrap();
                            plugin_lang = match file_name {
                                "requirements.txt" => {
                                    exec_path =
                                        Some(format!("{derived_root_path}/{derived_name}.py"));
                                    PluginLang::PyPip
                                }
                                "pyproject.toml" => {
                                    exec_path =
                                        Some(format!("{derived_root_path}/{derived_name}.py"));
                                    PluginLang::PyPoetry
                                }
                                // We dot have any information on standard pattern on where to find the
                                // plugin exec path, so for now we skip the indexing!
                                //
                                // N.B: The plugin should use the coffee manifest, period.
                                "go.mod" => PluginLang::Go,
                                "cargo.toml" => PluginLang::Rust,
                                "pubspec.yaml" => PluginLang::Dart,
                                "package.json" => PluginLang::JavaScript,
                                "tsconfig.json" => PluginLang::TypeScript,
                                _ => PluginLang::Unknown,
                            };
                            if plugin_lang != PluginLang::Unknown {
                                break;
                            }
                        }
                    }
                    debug!("possible plugin language: {:?}", plugin_lang);
                    if exec_path.is_none() {
                        let name = plugin_name.clone().unwrap();
                        log::warn!("we are not able to find the exec path for the plugin {name} written in {:?}, so we do not index it", plugin_lang);
                        log::info!("we are not able to detect the exec path for the plugin {name}");
                        continue;
                    }

                    let Some(exec_path) = exec_path else {
                        return Err(error!("exec path not known, but we should know at this point."));
                    };

                    debug!("exec path is {exec_path}");

                    // The language is already contained inside the configuration file.
                    let plugin = Plugin::new(
                        &plugin_name.unwrap(),
                        &root_path,
                        &exec_path,
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
                self.branch = if repo.find_branch("master", git2::BranchType::Local).is_ok() {
                    "master".to_owned()
                } else {
                    "main".to_owned()
                };
                let clone = clone_recursive_fix(repo, &self.url).await;
                self.index_repository().await?;
                clone
            }
            Err(err) => Err(CoffeeError::new(1, err.message())),
        }
    }

    async fn upgrade(
        &mut self,
        plugins: &Vec<Plugin>,
        branch: Option<String>,
    ) -> Result<CoffeeUpgrade, CoffeeError> {
        // get the list of the plugins installed from this repository
        // TODO: add a field of installed plugins in the repository struct instead
        let mut plugins_effected: Vec<String> = vec![];
        let remote_repo = self.list().await?;

        let plugins = plugins.clone();

        // FIXME: mark inside a repository what plugin is installed, and remove
        // this information from the configuration.
        for plugin in &remote_repo {
            if let Some(ind) = plugins
                .iter()
                .position(|elem| elem.name() == *plugin.name())
            {
                let plugin_name = &plugins[ind].name().clone();
                plugins_effected.push(plugin_name.to_owned());
            }
        }

        // Update the branch if it is specified
        if let Some(branch) = branch {
            self.branch = branch;
        }

        // pull the changes from the repository
        let status = fast_forward(&self.url.path_string, &self.branch)?;
        Ok(CoffeeUpgrade {
            repo: self.name(),
            status,
            plugins_effected,
        })
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
            branch: value.branch,
        }
    }
}

impl From<&StorageRepository> for Github {
    fn from(value: &StorageRepository) -> Self {
        Github {
            url: value.url.to_owned(),
            name: value.name.to_owned(),
            plugins: value.plugins.to_owned(),
            branch: value.branch.to_owned(),
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
            branch: value.branch,
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
            branch: value.branch.to_owned(),
        }
    }
}
