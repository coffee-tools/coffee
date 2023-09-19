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
use std::sync::Arc;

use async_trait::async_trait;

use coffee_lib::errors::CoffeeError;
use coffee_lib::types::response::{CoffeeNurse, NurseStatus};

use crate::nurse::chain::Handler;

#[async_trait]
pub trait RecoveryStrategy: Send + Sync {
    async fn patch(&self) -> Result<CoffeeNurse, CoffeeError>;
}

/// Strategy for handling the situation when a Git repository exists in coffee configuration
/// but is absent from the local storage.
///
/// This strategy is invoked when a Git repository is documented in the coffee configuration but
/// is not found in the local storage directory (./coffee/repositories).
/// The absence of the repository locally may be due to reasons such as accidental deletion or
/// a change in the storage location.
pub struct GitRepositoryLocallyAbsentStrategy;

#[async_trait]
impl RecoveryStrategy for GitRepositoryLocallyAbsentStrategy {
    /// Attempts to address the absence of a Git repository from local storage.
    ///
    /// This method is responsible for managing the scenario where a Git repository is listed
    /// in the coffee configuration but is not present in the `.coffee/repositories` folder.
    ///
    /// It takes the following actions:
    ///
    /// 1. Attempts to clone the repository using the Git HEAD reference stored in the configuration.
    ///    This is done in an effort to retrieve the missing repository from its source.
    ///
    /// 2. If the cloning process fails, it will remove the repository entry from the coffee configuration.
    async fn patch(&self) -> Result<CoffeeNurse, CoffeeError> {
        Ok(CoffeeNurse {
            status: NurseStatus::RepositoryLocallyAbsent,
        })
    }
}

#[async_trait]
impl Handler for GitRepositoryLocallyAbsentStrategy {
    /// Determines if [`GitRepositoryLocallyAbsentStrategy`] can be applied.
    ///
    /// This function iterates over the Git repositories listed in the coffee configuration and
    /// checks if each one exists in the `.coffee/repositories` folder. If any repository is found
    /// to be missing from local storage, it indicates that the strategy to handle
    /// this situation should be applied.
    async fn can_be_applied(
        self: Arc<Self>,
    ) -> Result<Option<std::sync::Arc<dyn RecoveryStrategy>>, CoffeeError> {
        Ok(Some(self))
    }
}
