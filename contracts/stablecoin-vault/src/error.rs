use thiserror::Error;

use cosmwasm_std::{StdError};
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum StableVaultError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No swaps can be performed in this pool")]
    NoSwapAvailabe {},

    #[error("Initialization values make no sense.")]
    InvalidInit {},

    #[error("Not enough funds to perform trade")]
    Broke {},
}