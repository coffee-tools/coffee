use async_trait::async_trait;
use reckless_lib::errors::RecklessError;
use reckless_lib::plugin::Plugin;
use reckless_lib::repository::Repository;

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
        Ok(())
    }

    /// list of the plugin installed inside the repository.
    ///
    /// M.B: in the future we want also list all the plugin installed
    /// inside the repository.
    async fn list(&self) -> Result<Vec<Plugin>, RecklessError> {
        Ok(vec![])
    }
}
