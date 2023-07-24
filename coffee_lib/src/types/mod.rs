//! Coffee Model Definition

// Definition of the request types.
pub mod request {
    use paperclip::actix::Apiv2Schema;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct Install {
        pub plugin: String,
        pub try_dynamic: bool,
    }

    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct Remove {
        pub plugin: String,
    }

    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct RemoteAdd {
        pub repository_name: String,
        pub repository_url: String,
    }

    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct RemoteRm {
        pub repository_name: String,
    }

    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct Show {
        pub plugin: String,
    }
}

// Definition of the response types.
pub mod response {
    use serde::{Deserialize, Serialize};

    use crate::plugin::Plugin;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CoffeeRemove {
        pub plugin: Plugin,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CoffeeList {
        pub plugins: Vec<Plugin>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CoffeeRemote {
        pub remotes: Option<Vec<CoffeeListRemote>>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CoffeeListRemote {
        pub local_name: String,
        pub url: String,
        pub plugins: Vec<Plugin>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum UpgradeStatus {
        UpToDate,
        Updated,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CoffeeUpgrade {
        pub repo: String,
        pub status: UpgradeStatus,
        /// If the status of the repository is
        /// alterate we return the list of plugin
        /// that are effected and need to be recompiled.
        pub plugins_effected: Vec<String>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CoffeeShow {
        pub readme: String,
    }
}
