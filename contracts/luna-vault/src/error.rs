use std::num::ParseIntError;
use thiserror::Error;

use cosmwasm_std::{DivideByZeroError, OverflowError, StdError};
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum LunaVaultError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("No swaps can be performed in this pool")]
    NoSwapAvailable {},

    #[error("The provided asset is not a native token.")]
    NotNativeToken {},

    #[error("The provided asset is not the luna token.")]
    NotLunaToken {},

    #[error("Not enough funds to perform trade")]
    Broke {},

    #[error("The provided fee is invalid")]
    InvalidFee {},

    #[error("The requesting contract is not whitelisted.")]
    NotWhitelisted {},

    #[error("The requesting contract already whitelisted.")]
    AlreadyWhitelisted {},

    #[error("The whitelist has reached its limit, can't store more contracts.")]
    WhitelistLimitReached {},

    #[error("You can not deposit into the vault during a flashloan.")]
    DepositDuringLoan {},

    #[error("Cancel losing trade.")]
    CancelLosingTrade {},

    #[error("Missing unbond data cache.")]
    UnbondHandlerMissingDataCache {},

    #[error("An error occurred reading the unbond data cache.")]
    UnbondDataCacheError {},

    #[error("The data parsed from the unbond handler instantiation msg and cache does not match.")]
    UnbondHandlerMismatchingDataCache {},

    #[error("There's no unbond handler assigned to the given address.")]
    NoUnbondHandlerAssigned {},

    #[error("The handler couldn't be released as it was not assigned the the given address.")]
    UnbondHandlerNotAssigned {},

    #[error("The handler triggering the release does not match the on.")]
    UnbondHandlerReleaseMismatch {},

    #[error("Expiration time couldn't be fetched.")]
    ExpirationTimeUnSet {},

    #[error("Couldn't get unbond handler.")]
    UnbondHandlerError {},

    #[error("Last balance is non-zero, you can only call this function once.")]
    Nonzero {},
}

impl From<semver::Error> for LunaVaultError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

impl LunaVaultError {
    pub fn generic_err(msg: impl Into<String>) -> Self {
        Self::Std(StdError::GenericErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        })
    }
}
