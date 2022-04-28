use cosmwasm_std::Response;

pub use crate::error::UnbondHandlerError;

pub mod contract;
mod error;
pub mod msg;
pub mod state;
mod queries;
mod commands;

type UnbondHandlerResult = Result<Response, UnbondHandlerError>;
