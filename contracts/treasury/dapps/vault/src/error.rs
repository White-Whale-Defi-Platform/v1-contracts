use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;
use white_whale::treasury::dapp_base::error::BaseDAppError;

#[derive(Error, Debug, PartialEq)]
pub enum VaultError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    BaseDAppError(#[from] BaseDAppError),

    #[error("This contract does not implement the cw20 swap function")]
    NoSwapAvailable {},

    #[error("The provided token: {} is not this vault's LP token", token)]
    NotLPToken { token: String},

    #[error("The provided token is not the base token")]
    WrongToken {},

    #[error("The actual amount of tokens transfered is different from the claimed amount.")]
    InvalidAmount {},
}
