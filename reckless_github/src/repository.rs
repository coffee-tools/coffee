use async_trait::async_trait;
use git2;
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
use walkdir::DirEntry;
use walkdir::WalkDir;

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

        // We are filtering to not iterate through files or any directories such as .git, .ci, .github
        for plugin_dir in WalkDir::new(repo_path)
            .max_depth(1)
            .into_iter()
            .filter_entry(|dir_entry| !is_hidden(dir_entry))
        {
            match plugin_dir {
                Ok(plugin_path) => {
                    let mut conf = None;
                    for file in WalkDir::new(plugin_path.path().to_str().unwrap()).max_depth(1) {
                        let file_dir = file.unwrap().clone();
                        let (path_to_plugin, plugin_name) =
                            get_plugin_info_from_path(file_dir.path()).unwrap();
                        let file_name = file_dir.file_name().to_str().unwrap();
                        let plugin_lang = match file_name {
                            "requirements.txt" => PluginLang::Python,
                            "go.mod" => PluginLang::Go,
                            "cargo.toml" => PluginLang::Rust,
                            "pubspec.yaml" => PluginLang::Dart,
                            "package.json" => PluginLang::JavaScript,
                            "tsconfig.json" => PluginLang::TypeScript,

                            "reckless.yml" | "reckless.yaml" => {
                                let conf_path = format!("{}/{}", path_to_plugin, file_name);
                                let mut conf_str = String::new();
                                File::open(conf_path.as_str())
                                    .await?
                                    .read_to_string(&mut conf_str)
                                    .await?;
                                conf = serde_yaml::from_str(&conf_str).unwrap();
                                continue;
                                // FIXME: store the conf inside the plugin
                            }
                            _ => continue,
                        };
                        debug!("PLUGIN: {} {}", plugin_name, path_to_plugin);
                        self.plugins.push(Plugin::new(
                            plugin_name.as_str(),
                            path_to_plugin.as_str(),
                            plugin_lang,
                            conf.clone(),
                        ));
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

    /// search inside the repository a plugin by name.
    fn get_plugin_by_name(&self, name: &str) -> Option<Plugin> {
        for plugin in &self.plugins {
            if plugin.name() == name {
                return Some(plugin.to_owned());
            }
        }
        None
    }
}
