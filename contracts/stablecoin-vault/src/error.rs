use thiserror::Error;

use cosmwasm_std::{OverflowError, StdError};
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum StableVaultError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("No swaps can be performed in this pool")]
    NoSwapAvailable {},

    #[error("Initialization values make no sense.")]
    InvalidInit {},

    #[error("Not enough funds to perform trade")]
    Broke {},

    #[error("The requesting contract is not whitelisted.")]
    NotWhitelisted {},

    #[error("The requesting contract already whitelisted.")]
    AlreadyWhitelisted {},

    #[error("You can not deposit into the vault during a flashloan.")]
    DepositDuringLoan {},

    #[error("Cancel losing trade.")]
    CancelLosingTrade {},

    #[error("Last balance is non-zero, you can only call this function once.")]
    Nonzero {},
}

impl From<semver::Error> for StableVaultError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
