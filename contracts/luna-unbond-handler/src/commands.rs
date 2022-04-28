use cosmwasm_std::{DepsMut, Env, from_binary, MessageInfo, Response, StdError, Uint128};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::AssetInfo;

use white_whale::anchor::{anchor_bluna_unbond_msg, anchor_withdraw_unbonded_msg};
use white_whale::memory::{ANCHOR_BLUNA_HUB_ID, BLUNA_ASSET_MEMORY_NAME};
use white_whale::memory::BLUNA_TOKEN_MEMORY_ID;
use white_whale::memory::queries::{query_asset_from_mem, query_contract_from_mem, query_contracts_from_mem};

use crate::{UnbondHandlerError, UnbondHandlerResult};
use crate::msg::Cw20HookMsg;
use crate::state::{ADMIN, STATE};

/// handler function invoked when the unbond handler contract receives
/// a transaction. This is triggered when someone wants to unbond and withdraw luna from the vault
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> UnbondHandlerResult {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Unbond {} => {
            let state = STATE.load(deps.storage)?;

            // only bluna token contract can execute this message
            let bluna_asset_info = query_asset_from_mem(deps.as_ref(), &state.memory_contract, BLUNA_TOKEN_MEMORY_ID)?;
            match bluna_asset_info {
                AssetInfo::NativeToken { .. } => return Err(UnbondHandlerError::UnsupportedToken {}),
                AssetInfo::Token { contract_addr } => {
                    if deps.api.addr_validate(&msg_info.sender.to_string())? != contract_addr {
                        return Err(UnbondHandlerError::UnsupportedToken {});
                    }
                }
            };

            unbond_bluna(deps, env, cw20_msg.amount, cw20_msg.sender)
        }
    }
}

fn unbond_bluna(
    deps: DepsMut,
    _env: Env,
    amount: Uint128,
    sender: String, // human who is requesting the unbonding
) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let contracts = [BLUNA_TOKEN_MEMORY_ID.to_string(), ANCHOR_BLUNA_HUB_ID.to_string(), ];
    let contract_addresses = query_contracts_from_mem(deps.as_ref(), &state.memory_contract, &contracts)?;

    if contract_addresses.len() != 2 {
        return Err(UnbondHandlerError::Std(StdError::generic_err("couldn't find contracts in memory")));
    }

    let bluna_address = contract_addresses.get(BLUNA_TOKEN_MEMORY_ID).ok_or(UnbondHandlerError::MemoryError(StdError::generic_err("couldn't find contracts in memory")))?.clone();
    let bluna_hub_address = contract_addresses.get(ANCHOR_BLUNA_HUB_ID).ok_or(UnbondHandlerError::MemoryError(StdError::generic_err("couldn't find contracts in memory")))?.clone();

    anchor_bluna_unbond_msg(bluna_address, bluna_hub_address, amount)
}

pub(crate) fn withdraw_unbonded_bluna(
    deps: DepsMut,
) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let bluna_hub_address = query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;
    anchor_withdraw_unbonded_msg(bluna_hub_address)

    //catch this stuff (or create a callback/submessage that will get triggered if this is successful)
    // then send it to state.owner
    // clear the owner and state.expiration_time`
}

/// Sets a new admin
pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> UnbondHandlerResult {
    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}
