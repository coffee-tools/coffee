use async_trait::async_trait;
use git2;
use log::debug;
use reckless_lib::errors::RecklessError;
use reckless_lib::plugin::Plugin;
use reckless_lib::repository::Repository;
use reckless_lib::utils::clone_recursive_fix;
use reckless_lib::utils::get_dir_path_from_url;

pub struct Github {
    /// the url of the repository to be able
    /// to get all the plugin information.
    url: String,
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
    pub fn new(name: &str, url: &str) -> Self {
        debug!("ADDING REPOSITORY: {name} {url}");
        Github {
            name: name.to_owned(),
            url: url.to_owned(),
            plugins: vec![],
        }
    }
}

#[async_trait]
impl Repository for Github {
    /// Init the repository where it is required to index
    /// all the plugin contained, and store somewhere the index.
    ///
    /// Where to store the index is an implementation
    /// details.
    async fn init(&self) -> Result<(), RecklessError> {
        debug!(
            "INITIALIZING REPOSITORY: {} {} > {}",
            self.name,
            self.url,
            get_dir_path_from_url(&self.url)
        );
        let res = git2::Repository::clone(&self.url, get_dir_path_from_url(&self.url));
        match res {
            Ok(repo) => clone_recursive_fix(repo, &self.url),
            Err(err) => Err(RecklessError::new(1, err.message())),
        }
    }

    /// list of the plugin installed inside the repository.
    ///
    /// M.B: in the future we want also list all the plugin installed
    /// inside the repository.
    async fn list(&self) -> Result<Vec<Plugin>, RecklessError> {
        Ok(vec![])
    }
}
