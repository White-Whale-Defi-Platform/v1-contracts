#![allow(unused_imports)]
#![allow(unused_variables)]

use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use white_whale::treasury::dapp_base::commands::{self as dapp_base_commands, handle_base_init};
use white_whale::treasury::dapp_base::common::BaseDAppResult;
use white_whale::treasury::dapp_base::msg::BaseInstantiateMsg;
use white_whale::treasury::dapp_base::queries as dapp_base_queries;
use white_whale::treasury::dapp_base::state::{BaseState, ADMIN, BASESTATE};
use white_whale::treasury::dapp_base::error::BaseDAppError;

use crate::commands;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub type BuyBackResult = Result<Response, BaseDAppError>;
use crate::state::{State, STATE};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> BaseDAppResult {
    let base_state = handle_base_init(deps.as_ref(), msg.base)?;

    let config: State = State {
        whale_vust_lp: msg.whale_vust_lp,
        vust_token: msg.vust_token,
        whale_token: msg.whale_token,
    };
    BASESTATE.save(deps.storage, &base_state)?;
    ADMIN.set(deps, Some(info.sender))?;
    STATE.save(deps.storage, &config)?;
    
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> BaseDAppResult {
    match msg {
        ExecuteMsg::Base(message) => dapp_base_commands::handle_base_message(deps, info, message),
        // handle dapp-specific messages here
        // ExecuteMsg::Custom{} => commands::custom_command(),
        ExecuteMsg::Buyback{ amount } => commands::handle_buyback_whale(deps, env, info, amount_to_buy),
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
