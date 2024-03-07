//! Nurse Chain of Responsibility rust implementation
//!
//! If you do not know what Chain Of Responsibility pattern
//! is, here is a small description:
//!
//! > Chain of Responsibility is behavioral design pattern
//! > that allows passing request along the chain of potential
//! > handlers until one of them handles request.
//! >
//! > The pattern allows multiple objects to handle the
//! > request without coupling sender class to the concrete
//! > classes of the receivers. The chain can be composed
//! > dynamically at runtime with any handler that follows
//! > a standard handler interface.
//!
//! In our case we do not need to handle a request, but we should
//! handler through the various recovery strategies to see what can
//! be applied.
//!
//! So in our case the handler is a specific recovery strategy
//! that tell the chain of responsibility if can be applied or not.
//!
//! If can be applied, the chain of responsibility will apply it.
//!
//! P.S: I do not know if my Advanced System programming teacher will be
//! proud of me for the following design, or simply mad with me!
//!
//! Author: Vincenzo Palazzo <vincenzopalazzo@member.fsf.org>
use std::sync::Arc;

use async_trait::async_trait;

use coffee_lib::errors::CoffeeError;
use coffee_lib::types::response::{ChainOfResponsibilityStatus, Defect};

use super::strategy::{CoffeeRepositoryDirCleanUp, GitRepositoryLocallyAbsentStrategy};
use crate::coffee::CoffeeManager;

#[async_trait]
pub trait Handler: Send + Sync {
    async fn can_be_applied(
        self: Arc<Self>,
        coffee: &CoffeeManager,
    ) -> Result<Option<Defect>, CoffeeError>;
}

pub struct RecoveryChainOfResponsibility {
    pub handlers: Vec<Arc<dyn Handler>>,
}

impl RecoveryChainOfResponsibility {
    /// Create a new instance of the chain of responsibility
    pub async fn new() -> Result<Self, CoffeeError> {
        Ok(Self {
            handlers: vec![
                Arc::new(GitRepositoryLocallyAbsentStrategy),
                Arc::new(CoffeeRepositoryDirCleanUp),
            ],
        })
    }

    /// Scan the chain of responsibility to see what can be applied
    /// and return the status of the chain of responsibility
    /// with the list of defects
    pub async fn scan(
        &self,
        coffee: &CoffeeManager,
    ) -> Result<ChainOfResponsibilityStatus, CoffeeError> {
        let mut defects: Vec<Defect> = vec![];
        for handler in self.handlers.iter() {
            if let Some(defect) = handler.clone().can_be_applied(coffee).await? {
                defects.push(defect);
            }
        }
        Ok(ChainOfResponsibilityStatus { defects })
    }
}
