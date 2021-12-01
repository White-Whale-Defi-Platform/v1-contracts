use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use white_whale::treasury::dapp_base::commands as dapp_base_commands;
use white_whale::treasury::dapp_base::common::DAppResult;
use white_whale::treasury::dapp_base::msg::{BaseInstantiateMsg, BaseQueryMsg};
use white_whale::treasury::dapp_base::queries as dapp_base_queries;
use white_whale::treasury::dapp_base::state::{State, ADMIN, STATE};

use crate::commands;
use crate::msg::ExecuteMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: BaseInstantiateMsg,
) -> DAppResult {
    let state = State {
        treasury_address: deps.api.addr_canonicalize(&msg.treasury_address)?,
        trader: deps.api.addr_canonicalize(&msg.trader)?,
    };

    // Store the initial config
    STATE.save(deps.storage, &state)?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DAppResult {
    match msg {
        ExecuteMsg::ProvideLiquidity {
            pool_id,
            main_asset_id,
            amount,
        } => commands::provide_liquidity(deps.as_ref(), info, main_asset_id, pool_id, amount),
        ExecuteMsg::WithdrawLiquidity {
            lp_token_id,
            amount,
        } => commands::withdraw_liquidity(deps.as_ref(), info, lp_token_id, amount),
        ExecuteMsg::SwapAsset {
            offer_id,
            pool_id,
            amount,
            max_spread,
            belief_price,
        } => commands::terraswap_swap(
            deps.as_ref(),
            env,
            info,
            offer_id,
            pool_id,
            amount,
            max_spread,
            belief_price,
        ),
        ExecuteMsg::Base(message) => dapp_base_commands::handle_base_message(deps, info, message),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: BaseQueryMsg) -> StdResult<Binary> {
    match msg {
        BaseQueryMsg::Config {} => to_binary(&dapp_base_queries::try_query_config(deps)?),
        BaseQueryMsg::AddressBook { id } => {
            to_binary(&dapp_base_queries::try_query_addressbook(deps, id)?)
        }
    }
}
