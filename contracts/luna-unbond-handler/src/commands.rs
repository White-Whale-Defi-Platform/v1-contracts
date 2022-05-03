use cosmwasm_std::{
    coins, from_binary, to_binary, BankMsg, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    Response, Uint128, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::querier::query_balance;

use white_whale::anchor::{anchor_bluna_unbond_msg, anchor_withdraw_unbonded_msg};
use white_whale::denom::LUNA_DENOM;
use white_whale::luna_vault::luna_unbond_handler::msg::{CallbackMsg, Cw20HookMsg};
use white_whale::luna_vault::msg::UnbondHandlerMsg;
use white_whale::luna_vault::queries::query_luna_vault_fees;
use white_whale::memory::error::MemoryError;
use white_whale::memory::queries::{
    query_asset_from_mem, query_contract_from_mem, query_contracts_from_mem,
};
use white_whale::memory::{ANCHOR_BLUNA_HUB_ID, BLUNA_TOKEN_MEMORY_ID, TREASURY_ADDRESS_ID};
use white_whale::query::anchor::query_unbond_requests;

use crate::serde_option::serde_option;
use crate::state::{ADMIN, STATE};
use crate::{UnbondHandlerError, UnbondHandlerResult};

/// Handler function invoked when the unbond handler contract receives
/// a transaction. This is triggered when someone wants to unbond and withdraw luna from the vault
pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
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
                    return Err(UnbondHandlerError::UnsupportedToken {});
                }
                AssetInfo::Token { contract_addr } => {
                    if deps.api.addr_validate(&msg_info.sender.to_string())? != contract_addr {
                        return Err(UnbondHandlerError::UnsupportedToken {});
                    }
                }
            };

            unbond_bluna(deps, cw20_msg.amount)
        }
    }
}

/// Triggers the unbonding process with the received bluna on Anchor
fn unbond_bluna(deps: DepsMut, amount: Uint128) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let contracts = [
        BLUNA_TOKEN_MEMORY_ID.to_string(),
        ANCHOR_BLUNA_HUB_ID.to_string(),
    ];
    let contract_addresses =
        query_contracts_from_mem(deps.as_ref(), &state.memory_contract, &contracts)?;

    // if the required contracts are not found in memory, return an error
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

    // create message for unbonding bluna on anchor
    let bluna_unbond_msg = anchor_bluna_unbond_msg(bluna_address, bluna_hub_address, amount)?;
    Ok(Response::new()
        .add_attributes(vec![
            ("action", "unbond_bluna"),
            ("amount", &amount.to_string()),
        ])
        .add_message(bluna_unbond_msg))
}

/// Withdraws the unbonded bluna from Anchor
pub fn withdraw_unbonded_bluna(deps: DepsMut, env: Env, info: MessageInfo) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let bluna_hub_address =
        query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;

    // create the message for withdrawing unbonded bluna from anchor
    let withdraw_unbonded_msg = anchor_withdraw_unbonded_msg(bluna_hub_address)?;

    // Callback for after withdrawing the unbonded bluna
    let after_withdraw_msg = CallbackMsg::AfterWithdraw {
        triggered_by_addr: info.sender.to_string(),
    }
    .to_cosmos_msg(&env.contract.address)?;

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

/// Handles callbacks from the contract
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
        CallbackMsg::AfterWithdraw { triggered_by_addr } => {
            after_withdraw(deps, env, triggered_by_addr)
        }
    }
}

/// Sends luna to its owner after it is withdrawn from Anchor, along with any liquidation fee
fn after_withdraw(deps: DepsMut, env: Env, triggered_by_addr: String) -> UnbondHandlerResult {
    let state = STATE.load(deps.storage)?;
    let triggered_by = deps.api.addr_validate(&triggered_by_addr)?;
    let owner = state
        .owner
        .clone()
        .ok_or(UnbondHandlerError::UnownedHandler {})?;

    let mut response = Response::new().add_attribute("action", "after_withdraw");

    // get amount of luna obtained from Anchor
    let refund_amount = query_balance(
        &deps.querier,
        env.contract.address.clone(),
        LUNA_DENOM.to_string(),
    )?;

    // if the withdrawal is done by someone other than the owner of the unbond handler AND past the expiration time, we charge a fee
    let expiration_time = state
        .expiration_time
        .ok_or(UnbondHandlerError::UnownedHandler {})?;

    // get treasury fee from luna vault, which is the admin of the unbond handler
    let luna_vault_addr = ADMIN
        .get(deps.as_ref())?
        .ok_or(UnbondHandlerError::NotAdminSet {})?;
    let vault_fees = query_luna_vault_fees(deps.as_ref(), &luna_vault_addr)?;
    let liquidation_fee_amount =
        match triggered_by != owner && env.block.time.seconds() > expiration_time {
            true => refund_amount * vault_fees.treasury_fee.share,
            false => Uint128::zero(),
        };

    // Construct refund message
    let refund_msg = Asset {
        info: AssetInfo::NativeToken {
            denom: LUNA_DENOM.to_string(),
        },
        amount: refund_amount.checked_sub(liquidation_fee_amount)?,
    }
    .into_msg(&deps.querier, owner)?;
    response = response.add_attribute(
        "refund_amount",
        refund_amount.checked_sub(liquidation_fee_amount)?,
    );
    response = response.add_attribute("liquidation_fee", liquidation_fee_amount);
    response = response.add_message(refund_msg);

    // Construct liquidation reward message if withdrawal wasn't triggered by the owner
    if !liquidation_fee_amount.is_zero() {
        // Send the liquidation_fee_amount * (1 - commission_fee) to whoever triggered the liquidation as a reward
        let reward_amount =
            liquidation_fee_amount * (Decimal::one() - vault_fees.commission_fee.share);
        let reward_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: triggered_by.to_string(),
            amount: coins(reward_amount.u128(), &*LUNA_DENOM.to_string()),
        });

        // Send the remaining chunk of the liquidation_fee_amount to the treasury
        let treasury_addr =
            query_contract_from_mem(deps.as_ref(), &state.memory_contract, TREASURY_ADDRESS_ID)?;
        let treasury_fee_amount = liquidation_fee_amount.checked_sub(reward_amount)?;
        let treasury_fee_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: treasury_addr.to_string(),
            amount: coins(treasury_fee_amount.u128(), &*LUNA_DENOM.to_string()),
        });

        response = response.add_attribute("reward_amount", reward_amount);
        response = response.add_attribute("treasury_fee_amount", treasury_fee_amount);

        response = response.add_messages(vec![reward_msg, treasury_fee_msg]);
    }

    // check if there are no more pending unbonds on Anchor for this handler, so that it can be reused
    let bluna_hub_address =
        query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;

    if query_unbond_requests(
        deps.as_ref(),
        bluna_hub_address,
        env.contract.address.clone(),
    )?
    .requests
    .is_empty()
    {
        // clean state so that the handler can be reused
        let mut state = state;
        state.owner = None;
        state.expiration_time = None;

        STATE.save(deps.storage, &state)?;

        // construct message for the vault to release the unbond handler
        let release_unbond_handler_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: luna_vault_addr.to_string(),
            msg: to_binary(&UnbondHandlerMsg::AfterUnbondHandlerReleased {
                unbond_handler_addr: env.contract.address.to_string(),
                previous_owner: owner.to_string(),
            })?,
            funds: vec![],
        });

        response = response.add_message(release_unbond_handler_msg);
        response = response.add_attribute("unbond_handler_released", true.to_string());
    }

    Ok(response)
}
