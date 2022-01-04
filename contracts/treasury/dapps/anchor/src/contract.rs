#![allow(unused_imports)]
#![allow(unused_variables)]

use cosmwasm_std::{Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Response, StdResult};

use white_whale::memory::item::Memory;
use white_whale::treasury::dapp_base::commands::{self as dapp_base_commands, handle_base_init};
use white_whale::treasury::dapp_base::common::BaseDAppResult;
use white_whale::treasury::dapp_base::msg::BaseInstantiateMsg;
use white_whale::treasury::dapp_base::queries as dapp_base_queries;
use white_whale::treasury::dapp_base::state::{ADMIN, BaseState};
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::state::BASESTATE;
use crate::commands;
use crate::error::AnchorError;
use crate::msg::{ExecuteMsg, QueryMsg};

pub type AnchorResult = Result<Response, BaseDAppError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: BaseInstantiateMsg,
) -> BaseDAppResult {
    let base_state = handle_base_init(deps.as_ref(), msg)?;

    BASESTATE.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> BaseDAppResult {
    match msg {
        ExecuteMsg::Base(message) => dapp_base_commands::handle_base_message(deps, info, message),
        // handle dapp-specific messages here
        // ExecuteMsg::Custom{} => commands::custom_command(),
        ExecuteMsg::DepositStable{ deposit_amount } => commands::handle_deposit_stable(deps.as_ref(), env, info, deposit_amount),
        ExecuteMsg::RedeemStable{ withdraw_amount } => commands::handle_redeem_stable(deps.as_ref(), env, info, withdraw_amount)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => dapp_base_queries::handle_base_query(deps, message),
        // handle dapp-specific queries here
        // QueryMsg::Custom{} => queries::custom_query(),
    }
}
