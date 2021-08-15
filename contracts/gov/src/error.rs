use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Must deposit more than {0} token")]
    InsufficientProposalDeposit(u128),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Poll is not in progress")]
    PollNotInProgress {},

    #[error("Voting period has not expired")]
    PollVotingPeriod {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
