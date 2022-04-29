use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;
use white_whale::memory::error::MemoryError;

#[derive(Error, Debug, PartialEq)]
pub enum UnbondHandlerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("Unsupported token")]
    UnsupportedToken {},

    #[error("{0}")]
    MemoryError(#[from] MemoryError),
}

impl From<semver::Error> for UnbondHandlerError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
