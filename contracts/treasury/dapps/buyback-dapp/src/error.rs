use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;
use white_whale::treasury::dapp_base::error::BaseDAppError;

#[derive(Error, Debug, PartialEq)]
pub enum BuyBackError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    BaseDAppError(#[from] BaseDAppError),

    #[error("Not enough funds to perform buyback")]
    NotEnoughFunds {},
}
