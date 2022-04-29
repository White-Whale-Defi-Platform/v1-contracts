use std::str::FromStr;

use cosmwasm_std::{
    from_binary, Addr, BankMsg, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError,
    SubMsg, Uint128,
};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::querier::query_balance;

use white_whale::anchor::{anchor_bluna_unbond_msg, anchor_withdraw_unbonded_msg};
use white_whale::denom::LUNA_DENOM;
use white_whale::memory::error::MemoryError;
use white_whale::memory::queries::{
    query_asset_from_mem, query_contract_from_mem, query_contracts_from_mem,
};
use white_whale::memory::{ANCHOR_BLUNA_HUB_ID, BLUNA_TOKEN_MEMORY_ID};
use white_whale::query::anchor::query_unbond_requests;

use crate::msg::{CallbackMsg, Cw20HookMsg};
use crate::serde_option::serde_option;
use crate::state::{ADMIN, STATE};
use crate::{UnbondHandlerError, UnbondHandlerResult};

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
            let bluna_asset_info =
                query_asset_from_mem(deps.as_ref(), &state.memory_contract, BLUNA_TOKEN_MEMORY_ID)?;
            match bluna_asset_info {
                AssetInfo::NativeToken { .. } => {
                    return Err(UnbondHandlerError::UnsupportedToken {})
                }
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

fn unbond_bluna(deps: DepsMut, _env: Env, amount: Uint128) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let contracts = [
        BLUNA_TOKEN_MEMORY_ID.to_string(),
        ANCHOR_BLUNA_HUB_ID.to_string(),
    ];
    let contract_addresses =
        query_contracts_from_mem(deps.as_ref(), &state.memory_contract, &contracts)?;

    if contract_addresses.len() != 2 {
        return Err(UnbondHandlerError::MemoryError(
            MemoryError::NotFoundInMemory {},
        ));
    }

    let bluna_address = contract_addresses
        .get(BLUNA_TOKEN_MEMORY_ID)
        .ok_or(UnbondHandlerError::MemoryError(
            MemoryError::NotFoundInMemory {},
        ))?
        .clone();
    let bluna_hub_address = contract_addresses
        .get(ANCHOR_BLUNA_HUB_ID)
        .ok_or(UnbondHandlerError::MemoryError(
            MemoryError::NotFoundInMemory {},
        ))?
        .clone();

    let bluna_unbond_msg = anchor_bluna_unbond_msg(bluna_address, bluna_hub_address, amount)?;
    Ok(Response::new()
        .add_attributes(vec![
            ("action", "unbond_bluna"),
            ("amount", &amount.to_string()),
        ])
        .add_message(bluna_unbond_msg))
}

pub(crate) fn withdraw_unbonded_bluna(deps: DepsMut, env: Env) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let bluna_hub_address =
        query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;

    let withdraw_unbonded_msg = anchor_withdraw_unbonded_msg(bluna_hub_address)?;

    // Callback for after withdrawing the unbonded bluna
    let after_withdraw_msg = CallbackMsg::AfterWithdraw {}.to_cosmos_msg(&env.contract.address)?;

    Ok(Response::new()
        .add_attributes(vec![("action", "withdraw_unbonded_bluna")])
        .add_messages(vec![withdraw_unbonded_msg, after_withdraw_msg]))
}

/// Sets a new admin
pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> UnbondHandlerResult {
    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?;
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;

    Ok(Response::default()
        .add_attribute("previous_admin", serde_option(previous_admin))
        .add_attribute("new_admin", admin))
}

/// Updates the state of the contract
pub fn update_state(
    deps: DepsMut,
    msg_info: MessageInfo,
    owner: Option<String>,
    expiration_time: Option<u64>,
    memory_contract: Option<String>,
) -> UnbondHandlerResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;

    let mut attrs = vec![];

    if let Some(owner) = owner {
        state.owner = Some(deps.api.addr_validate(&owner)?);
        attrs.push(("new_owner", owner));
    }

    if let Some(expiration_time) = expiration_time {
        state.expiration_time = Some(expiration_time);
        attrs.push(("new_expiration_time", expiration_time.to_string()));
    }

    if let Some(memory_contract) = memory_contract {
        state.memory_contract = deps.api.addr_validate(&memory_contract)?;
        attrs.push(("new_memory_contract", memory_contract));
    }

    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attributes(attrs))
}

pub(crate) fn handle_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CallbackMsg,
) -> UnbondHandlerResult {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(UnbondHandlerError::NotCallback {});
    }

    match msg {
        CallbackMsg::AfterWithdraw {} => after_withdraw(deps, env, info),
    }
}

/// Sends luna to its owner after it is withdrawn from Anchor.
fn after_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let owner = state
        .owner
        .clone()
        .ok_or(UnbondHandlerError::UnownedHandler {})?;

    let refund_amount = query_balance(
        &deps.querier,
        env.contract.address.clone(),
        LUNA_DENOM.to_string(),
    )?;

    // if the withdrawal is done by someone other than the owner of the unbond handler AND past the expiration time, we charge a fee
    let expiration_time = state
        .expiration_time
        .ok_or(UnbondHandlerError::UnownedHandler {})?;

    // todo: query the luna vault for the proper fee rate
    let fee_amt = match info.sender != owner && env.block.time.seconds() > expiration_time {
        true => refund_amount * Decimal::from_str("0.01")?,
        false => Uint128::zero(),
    };

    // Construct refund message
    let refund_msg = Asset {
        info: AssetInfo::NativeToken {
            denom: LUNA_DENOM.to_string(),
        },
        amount: refund_amount.checked_sub(fee_amt)?,
    }
    .into_msg(&deps.querier, owner)?;

    // todo: construct message to
    // 1. send fee_amt * (1 - VAULT_FEE_RATE) to info.sender (if it is non-zero)
    // 2. construct message to send fee_amt * VAULT_FEE_RATE to the vault

    // check if there is no more unbonds on Anchor, so that the handler can be reused
    // todo: ensure that user has no waitlist requests for unbonding
    // maybe we need to store a counter of the amount of unbond requests, and decrement to 0 eventually?
    let bluna_hub_address =
        query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;

    if query_unbond_requests(deps.as_ref(), bluna_hub_address, env.contract.address)?
        .requests
        .is_empty()
    {
        // clean state so that the handler can be reused
        let mut state = state;
        state.owner = None;
        state.expiration_time = None;

        STATE.save(deps.storage, &state)?;
    }

    Ok(Response::new()
        .add_attribute("action", "after_withdraw")
        .add_message(refund_msg))
}
