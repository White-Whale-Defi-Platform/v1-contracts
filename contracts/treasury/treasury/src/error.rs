use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TreasuryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Trader is already whitelisted")]
    AlreadyInList {},

    #[error("Trader not found in whitelist")]
    NotInList {},

    #[error("Sender is not whitelisted")]
    SenderNotWhitelisted {},
}
