use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use white_whale::treasury::dapp_base::commands as dapp_base_commands;
use white_whale::treasury::dapp_base::common::DAppResult;
use white_whale::treasury::dapp_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use white_whale::treasury::dapp_base::queries as dapp_base_queries;
use white_whale::treasury::dapp_base::state::{State, ADMIN, STATE};

use crate::commands;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> DAppResult {
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
        ExecuteMsg::UpdateConfig {
            treasury_address,
            trader,
        } => dapp_base_commands::update_config(deps, info, treasury_address, trader),
        ExecuteMsg::SetAdmin { admin } => {
            ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
            Ok(Response::default()
                .add_attribute("previous admin", previous_admin)
                .add_attribute("admin", admin))
        }
        ExecuteMsg::UpdateAddressBook { to_add, to_remove } => {
            dapp_base_commands::update_address_book(deps, info, to_add, to_remove)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&dapp_base_queries::try_query_config(deps)?),
        QueryMsg::AddressBook { id } => {
            to_binary(&dapp_base_queries::try_query_addressbook(deps, id)?)
        }
    }
}
