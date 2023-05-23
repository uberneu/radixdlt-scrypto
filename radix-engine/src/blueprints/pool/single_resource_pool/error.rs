use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use radix_engine_common::types::*;
use radix_engine_common::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SingleResourcePoolError {
    CantCreateAPoolOfNonFungibleResources { resource_address: ResourceAddress },
    IllegalState,
}

impl From<SingleResourcePoolError> for RuntimeError {
    fn from(error: SingleResourcePoolError) -> Self {
        Self::ApplicationError(ApplicationError::SingleResourcePoolError(error))
    }
}
