use cosmwasm_std::{OverflowError, StdError};
use cw_controllers::AdminError;
use thiserror::Error;
use white_whale::memory::error::MemoryError;

#[derive(Error, Debug, PartialEq)]
pub enum UnbondHandlerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("The contract has not set the luna vault address")]
    NotAdminSet {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("Unsupported token")]
    UnsupportedToken {},

    #[error("Contract is not owned by any address")]
    UnownedHandler {},

    #[error("Expiration time was impossible to calculate")]
    WrongExpirationTime {},

    #[error("{0}")]
    MemoryError(#[from] MemoryError),
}

impl From<semver::Error> for UnbondHandlerError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
