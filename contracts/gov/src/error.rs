use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Must deposit more than {0} token")]
    InsufficientProposalDeposit(u128),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot make a text proposal to expired state")]
    NoExecuteData {},

    #[error("Poll is not in progress")]
    PollNotInProgress {},

    #[error("Poll is not in passed status")]
    PollNotPassed {},

    #[error("Voting period has not expired")]
    PollVotingPeriod {},

    #[error("Timelock period has not expired")]
    TimelockNotExpired {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
