use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TreasuryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Cannot spend more than spend_limit")]
    TooMuchSpend {},

    #[error("Trader already added")]
    AlreadyInList {},

    #[error("Trader not found in list")]
    NotInList {},

    #[error("Sender is not whitelisted")]
    SenderNotWhitelisted {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
