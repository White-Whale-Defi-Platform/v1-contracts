use cosmwasm_std::{Addr, BankMsg, CosmosMsg, DepsMut, Env, from_binary, MessageInfo, Response, StdError, SubMsg, Uint128};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::querier::query_balance;

use white_whale::anchor::{anchor_bluna_unbond_msg, anchor_withdraw_unbonded_msg};
use white_whale::denom::LUNA_DENOM;
use white_whale::memory::{ANCHOR_BLUNA_HUB_ID, BLUNA_ASSET_MEMORY_NAME};
use white_whale::memory::BLUNA_TOKEN_MEMORY_ID;
use white_whale::memory::queries::{query_asset_from_mem, query_contract_from_mem, query_contracts_from_mem};
use white_whale::ust_vault::msg::ExecuteMsg::Callback;

use crate::{UnbondHandlerError, UnbondHandlerResult};
use crate::msg::{CallbackMsg, Cw20HookMsg};
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

            unbond_bluna(deps, env, cw20_msg.amount)
        }
    }
}

fn unbond_bluna(
    deps: DepsMut,
    _env: Env,
    amount: Uint128,
) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let contracts = [BLUNA_TOKEN_MEMORY_ID.to_string(), ANCHOR_BLUNA_HUB_ID.to_string(), ];
    let contract_addresses = query_contracts_from_mem(deps.as_ref(), &state.memory_contract, &contracts)?;

    if contract_addresses.len() != 2 {
        return Err(UnbondHandlerError::Std(StdError::generic_err("couldn't find contracts in memory")));
    }

    let bluna_address = contract_addresses.get(BLUNA_TOKEN_MEMORY_ID).ok_or(UnbondHandlerError::MemoryError(StdError::generic_err("couldn't find contracts in memory")))?.clone();
    let bluna_hub_address = contract_addresses.get(ANCHOR_BLUNA_HUB_ID).ok_or(UnbondHandlerError::MemoryError(StdError::generic_err("couldn't find contracts in memory")))?.clone();

    let bluna_unbond_msg = anchor_bluna_unbond_msg(bluna_address, bluna_hub_address, amount);
    Ok(Response::new()
        .add_attributes(vec![
            ("action", "unbond_bluna"),
            ("amount", amount.to_string()),
        ])
        .add_message(bluna_unbond_msg))
}

pub(crate) fn withdraw_unbonded_bluna(
    deps: DepsMut,
) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let bluna_hub_address = query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;

    let withdraw_unbonded_msg = SubMsg::new(anchor_withdraw_unbonded_msg(bluna_hub_address));

    // Callback for after withdrawing the unbonded bluna
    let after_withdraw_msg = CallbackMsg::AfterWithdraw {}.to_cosmos_msg(&env.contract.address)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "withdraw_unbonded_bluna"),
            ("amount", amount.to_string()),
        ])
        .add_submessage(withdraw_unbonded_msg)
        .add_message(after_withdraw_msg))
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

/// Updates the state of the contract
pub fn update_state(deps: DepsMut, msg_info: MessageInfo, owner: Option<String>, expiration_time: Option<u64>, memory_contract: Option<String>) -> UnbondHandlerResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;

    let mut attrs = vec![];

    if let Some(owner) = owner {
        state.owner = Some(deps.api.addr_validate(&owner)?);
        attrs.push(("new owner", owner));
    }

    if let Some(expiration_time) = expiration_time {
        state.expiration_time = Some(expiration_time);
        attrs.push(("new expiration_time", expiration_time.to_string()));
    }

    if let Some(memory_contract) = memory_contract {
        state.memory_contract = deps.api.addr_validate(&memory_contract)?;
        attrs.push(("new memory_contract", memory_contract));
    }

    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attributes(attrs))
}

pub(crate) fn _handle_callback(deps: DepsMut, env: Env, info: MessageInfo, msg: CallbackMsg) -> UnbondHandlerResult {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(UnbondHandlerError::NotCallback {});
    }
    match msg {
        CallbackMsg::AfterWithdraw {} => _after_withdraw(deps, env),
    }
}

/// Sends luna to its owner after it is withdrawn from Anchor.
fn _after_withdraw(deps: DepsMut, env: Env) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let owner = state.owner?;

    let refund_amount = query_balance(&deps.querier, env.contract.address, LUNA_DENOM.to_string())?;
    // Construct refund message
    let refund_asset = Asset {
        info: AssetInfo::NativeToken { denom: LUNA_DENOM.to_string() },
        amount: refund_amount,
    };
    let taxed_asset = refund_asset.deduct_tax(&deps.querier)?;

    let refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: owner.to_string(),
        amount: vec![taxed_asset],
    });

    //todo check for further unbonding batches on anchor
    // Clean state so that the handler can be reused
    let mut state = STATE.load(deps.storage)?;
    state.owner = None;
    state.expiration_time = None;

    STATE.save(deps.storage, &state);

    Ok(Response::new()
        .add_attribute("action", "after_withdraw")
        .add_message(refund_msg))
}
