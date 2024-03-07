//! Nurse Strategy patter implementation for recovery methods.
//!
//! If you do not know what Strategy patter is here is a small
//! description.
//!
//! >The basic idea behind the Strategy pattern is that, given an
//! > algorithm solving a particular problem,  we define only
//! > the skeleton of the algorithm at an abstract level, and we
//! > separate the specific algorithm’s implementation into
//! > different parts.
//! >
//! > In this way, a client using the algorithm may choose
//! > a specific implementation, while the general algorithm
//! > workflow remains the same. In other words, the abstract
//! > specification of the class does not depend on the specific
//! > implementation of the derived class, but specific implementation
//! > must adhere to the abstract specification.
//!
//! So in this specific case the nurse command may need
//! different kind of recovery algorithm, so we should
//! be able to choose the algorithm at runtime.
//!
//! Author: Vincenzo Palazzo <vincenzopalazzo@member.fsf.org>
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use coffee_lib::errors::CoffeeError;
use coffee_lib::types::response::Defect;

use crate::coffee::CoffeeManager;
use crate::nurse::chain::Handler;

/// Strategy for handling the situation when a Git repository exists in coffee configuration
/// but is absent from the local storage.
///
/// This strategy is invoked when a Git repository is documented in the coffee configuration but
/// is not found in the local storage directory (./coffee/repositories).
/// The absence of the repository locally may be due to reasons such as accidental deletion or
/// a change in the storage location.
pub struct GitRepositoryLocallyAbsentStrategy;

#[async_trait]
impl Handler for GitRepositoryLocallyAbsentStrategy {
    /// Determines if a repository is missing from local storage.
    ///
    /// This function iterates over the Git repositories listed in the coffee configuration and
    /// checks if each one exists in the `.coffee/repositories` folder. If any repository is found
    /// to be missing from local storage, it indicates that the strategy to handle
    /// this situation should be applied.
    async fn can_be_applied(
        self: Arc<Self>,
        coffee: &CoffeeManager,
    ) -> Result<Option<Defect>, CoffeeError> {
        let mut repos: Vec<String> = Vec::new();
        let coffee_repos = &coffee.repos;
        for repo in coffee_repos.values() {
            log::debug!("Checking if repository {} exists locally", repo.name());
            let repo_path = repo.url().path_string;
            let repo_path = Path::new(&repo_path);
            if !repo_path.exists() {
                log::debug!("Repository {} is missing locally", repo.name());
                repos.push(repo.name().to_string());
            }
        }

        if repos.is_empty() {
            log::debug!("No repositories missing locally");
            Ok(None)
        } else {
            log::debug!("Found {} repositories missing locally", repos.len());
            Ok(Some(Defect::RepositoryLocallyAbsent(repos)))
        }
    }
}

/// Stategy for migration of the repository global directory
/// to a local directory for each network, see [1]
///
/// This is a strategy tht return the list of network that
/// needs to be migrated to to the new repository configuration.
///
/// [1] https://github.com/coffee-tools/coffee/issues/234
pub struct CoffeeRepositoryDirCleanUp;

#[async_trait]
impl Handler for CoffeeRepositoryDirCleanUp {
    async fn can_be_applied(
        self: Arc<Self>,
        coffee: &CoffeeManager,
    ) -> Result<Option<Defect>, CoffeeError> {
        let networks = ["testnet", "signet", "bitcoin", "liquid"];
        // Check whether there exists a network-specific repositories folder for each network.
        let mut directory_moving = vec![];
        for network in networks {
            let subpath_repo = format!("{}/{network}/repositories", coffee.config.root_path);
            if !Path::exists(Path::new(&subpath_repo)) {
                directory_moving.push((network.to_string(), subpath_repo));
            }
        }
        if directory_moving.is_empty() {
            return Ok(None);
        }
        Ok(Some(Defect::CoffeeGlobalrepoCleanup(directory_moving)))
    }
}
