//! Coffee Model Definition

// Definition of the request types.
pub mod request {
    #[cfg(feature = "open-api")]
    use paperclip::actix::Apiv2Schema;
    use serde::{Deserialize, Serialize};

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
    pub struct RemotePluginsList {
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
    use std::fmt;

    use serde::{Deserialize, Serialize};

    use crate::plugin::Plugin;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CoffeeRemove {
        pub plugin: Plugin,
    }

    // This struct is used to represent the list of plugins
    // that are installed in the coffee configuration
    // or the list of plugins that are available in a remote
    // repository.
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
        pub commit_id: Option<String>,
        pub date: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum UpgradeStatus {
        /// CommitId, Date
        UpToDate(String, String),
        /// CommitId, Date
        Updated(String, String),
    }

    impl UpgradeStatus {
        pub fn date(&self) -> String {
            match self {
                UpgradeStatus::UpToDate(_, date) => date.clone(),
                UpgradeStatus::Updated(_, date) => date.clone(),
            }
        }
        pub fn commit_id(&self) -> String {
            match self {
                UpgradeStatus::UpToDate(commit_id, _) => commit_id.clone(),
                UpgradeStatus::Updated(commit_id, _) => commit_id.clone(),
            }
        }
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

    /// This struct is used to represent a defect
    /// that can be patched by the nurse.
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum Defect {
        // A patch operation when a git repository is present in the coffee configuration
        // but is absent from the local storage.
        RepositoryLocallyAbsent(Vec<String>),
        // TODO: Add more patch operations
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ChainOfResponsibilityStatus {
        pub defects: Vec<Defect>,
    }

    impl ChainOfResponsibilityStatus {
        pub fn is_sane(&self) -> bool {
            self.defects.is_empty()
        }
    }

    impl fmt::Display for ChainOfResponsibilityStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.defects.is_empty() {
                write!(f, "Coffee is sane")
            } else {
                writeln!(f, "Coffee has the following defects:")?;
                for (i, defect) in self.defects.iter().enumerate() {
                    match defect {
                        Defect::RepositoryLocallyAbsent(repos) => {
                            write!(f, "{}. Repository missing locally: ", i + 1)?;
                            for repo in repos {
                                write!(f, " {}", repo)?;
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    /// This struct is used to represent the status of nurse,
    /// either sane or not.
    /// If not sane, return the action that nurse has taken.
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum NurseStatus {
        RepositoryLocallyRestored(Vec<String>),
        RepositoryLocallyRemoved(Vec<String>),
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CoffeeNurse {
        pub status: Vec<NurseStatus>,
    }

    impl CoffeeNurse {
        pub fn is_sane(&self) -> bool {
            self.status.is_empty()
        }

        pub fn organize(&mut self) {
            // For every action taken by the nurse, we want to
            // have 1 entry with the list of repositories affected.
            let mut new_status: Vec<NurseStatus> = vec![];
            let mut repositories_locally_removed: Vec<String> = vec![];
            let mut repositories_locally_restored: Vec<String> = vec![];
            for repo in self.status.iter() {
                match repo {
                    NurseStatus::RepositoryLocallyRemoved(repos) => {
                        repositories_locally_removed.append(&mut repos.clone())
                    }
                    NurseStatus::RepositoryLocallyRestored(repos) => {
                        repositories_locally_restored.append(&mut repos.clone())
                    }
                }
            }
            if !repositories_locally_removed.is_empty() {
                new_status.push(NurseStatus::RepositoryLocallyRemoved(
                    repositories_locally_removed,
                ));
            }
            if !repositories_locally_restored.is_empty() {
                new_status.push(NurseStatus::RepositoryLocallyRestored(
                    repositories_locally_restored,
                ));
            }
            self.status = new_status;
        }
    }

    impl fmt::Display for NurseStatus {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                NurseStatus::RepositoryLocallyRestored(val) => {
                    write!(f, "Repositories restored locally: {}", val.join(" "))
                }
                NurseStatus::RepositoryLocallyRemoved(val) => {
                    write!(f, "Repositories removed locally: {}", val.join(" "))
                }
            }
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CoffeeTip {
        pub for_plugin: String,
        pub invoice: String,
        pub status: String,
        pub destination: Option<String>,
        pub amount_msat: u64,
        // This includes the fee
        pub amount_sent_msat: u64,
        pub warning_partial_completion: Option<String>,
    }
}
