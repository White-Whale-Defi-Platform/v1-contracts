use thiserror::Error;

use cosmwasm_std::StdError;
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum CommunityFundError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Not enough funds.")]
    NotEnoughFunds {},

    #[error("Too many tokens. Deposit only accepts WHALE.")]
    WrongDepositTooManyTokens {},

    #[error("Deposit only accepts WHALE.")]
    WrongDepositToken {},
}
