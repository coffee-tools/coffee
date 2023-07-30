//! Coffee Model Definition

// Definition of the request types.
pub mod request {
    #[cfg(feature = "open-api")]
    use paperclip::actix::Apiv2Schema;
    use serde::{Deserialize, Serialize};

    #[cfg(not(feature = "open-api"))]
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Install {
        pub plugin: String,
        pub try_dynamic: bool,
    }

    #[cfg(not(feature = "open-api"))]
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Remove {
        pub plugin: String,
    }

    #[cfg(not(feature = "open-api"))]
    #[derive(Debug, Deserialize, Serialize)]
    pub struct RemoteAdd {
        pub repository_name: String,
        pub repository_url: String,
    }

    #[cfg(not(feature = "open-api"))]
    #[derive(Debug, Deserialize, Serialize)]
    pub struct RemoteRm {
        pub repository_name: String,
    }

    #[cfg(not(feature = "open-api"))]
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Show {
        pub plugin: String,
    }

    #[cfg(not(feature = "open-api"))]
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Search {
        pub plugin: String,
    }

    #[cfg(feature = "open-api")]
    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct Install {
        pub plugin: String,
        pub try_dynamic: bool,
    }

    #[cfg(feature = "open-api")]
    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct Remove {
        pub plugin: String,
    }

    #[cfg(feature = "open-api")]
    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct RemoteAdd {
        pub repository_name: String,
        pub repository_url: String,
    }

    #[cfg(feature = "open-api")]
    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct RemoteRm {
        pub repository_name: String,
    }

    #[cfg(feature = "open-api")]
    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct Show {
        pub plugin: String,
    }

    #[cfg(feature = "open-api")]
    #[derive(Debug, Deserialize, Apiv2Schema, Serialize)]
    pub struct Search {
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

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CoffeeSearch {
        pub repository_url: String,
        pub plugin: Plugin,
    }
}
