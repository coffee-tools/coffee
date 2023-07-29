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

use crate::nurse::chain::Handler;

#[async_trait]
pub trait RecoveryStrategy: Send + Sync {
    async fn patch(&self) -> Result<(), CoffeeError>;
}

pub struct GitRepositoryMissedStrategy;

#[async_trait]
impl RecoveryStrategy for GitRepositoryMissedStrategy {
    async fn patch(&self) -> Result<(), CoffeeError> {
        unimplemented!()
    }
}

#[async_trait]
impl Handler for GitRepositoryMissedStrategy {
    async fn can_be_apply(
        self: Arc<Self>,
    ) -> Result<Option<std::sync::Arc<dyn RecoveryStrategy>>, CoffeeError> {
        Ok(Some(self))
    }
}
