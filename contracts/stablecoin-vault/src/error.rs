use thiserror::Error;

use cosmwasm_std::{StdError};

#[derive(Error, Debug, PartialEq)]
pub enum StableVaultError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("no swaps can be performed in this pool")]
    NoSwapAvailabe {},
}