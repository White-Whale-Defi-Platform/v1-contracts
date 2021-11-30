use thiserror::Error;

use cosmwasm_std::{Response, StdError};
use cw_controllers::AdminError;
use serde_json::Error as SerdeError;

#[derive(Error, Debug)]
pub enum FederatorError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Serde Error: {0}")]
    Serde(#[from] SerdeError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("Not enough funds to perform arb-trade")]
    Broke {},
}
