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

    #[error("no swaps can be performed in this pool")]
    NoSwapAvailabe {},
}