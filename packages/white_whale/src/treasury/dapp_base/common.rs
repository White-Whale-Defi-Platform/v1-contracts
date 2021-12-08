use cosmwasm_std::Response;
use crate::treasury::dapp_base::error::BaseDAppError;

pub const PAIR_POSTFIX: &str = "_pair";

pub type DAppResult = Result<Response, BaseDAppError>;
