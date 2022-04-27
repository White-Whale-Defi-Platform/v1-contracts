use core::result::Result::Err;
use cosmwasm_std::{CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg};
use terraswap::asset::{Asset, AssetInfo};
use white_whale::denom::LUNA_DENOM;
use white_whale::luna_vault::msg::{CallbackMsg, FlashLoanPayload};
use white_whale::tax::into_msg_without_tax;

use crate::commands::{deposit_passive_strategy, withdraw_passive_strategy};
use crate::contract::VaultResult;
use crate::error::LunaVaultError;
use crate::helpers::{compute_total_value, get_lp_token_address};
use crate::pool_info::PoolInfoRaw;
use crate::state::{DEPOSIT_INFO, FEE, POOL_INFO, PROFIT, STATE};
const ROUNDING_ERR_COMPENSATION: u32 = 10u32;

pub fn handle_flashloan(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payload: FlashLoanPayload,
) -> VaultResult<Response> {
    let state = STATE.load(deps.storage)?;
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    let fees = FEE.load(deps.storage)?;
    let whitelisted_contracts = state.whitelisted_contracts;
    let whitelisted: bool;
    // Check if requested asset is base token of vault
    deposit_info.assert(&payload.requested_asset.info)?;

    // Check if sender is whitelisted
    if !whitelisted_contracts.contains(&deps.api.addr_validate(&info.sender.to_string())?) {
        // Check if non-whitelisted are allowed to borrow
        if state.allow_non_whitelisted {
            whitelisted = false;
        } else {
            return Err(LunaVaultError::NotWhitelisted {});
        }
    } else {
        whitelisted = true;
    }

    // Do we have enough funds?
    let pool_info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (total_value, luna_available, _, _, _) =
        compute_total_value(&env, deps.as_ref(), &pool_info)?;
    let requested_asset = payload.requested_asset;

    // Max tax buffer will be 2 transfers of the borrowed assets
    // Passive Strategy -> Vault -> Caller
    let tax_buffer = Uint128::from(2u32) * requested_asset.compute_tax(&deps.querier)?
        + Uint128::from(ROUNDING_ERR_COMPENSATION);

    if total_value < requested_asset.amount + tax_buffer {
        return Err(LunaVaultError::Broke {});
    }
    // Init response
    let mut response = Response::new().add_attribute("Action", "Flashloan");

    //TODO
    // Withdraw funds from Passive Strategy if needed
    // FEE_BUFFER as buffer for fees and taxes
    /*    if (requested_asset.amount + tax_buffer) > luna_available {
        // Attempt to remove some money from anchor
        let to_withdraw = (requested_asset.amount + tax_buffer) - luna_available;
        let aust_exchange_rate = query_aust_exchange_rate(
            env.clone(),
            deps.as_ref(),
            state.anchor_money_market_address.to_string(),
        )?;

        let withdraw_msg = anchor_withdraw_msg(
            state.bluna_address,
            state.anchor_money_market_address,
            to_withdraw * aust_exchange_rate.inv().unwrap(),
        )?;

        // Add msg to response and update withdrawn value
        response = response
            .add_message(withdraw_msg)
            .add_attribute("Anchor withdrawal", to_withdraw.to_string())
            .add_attribute("ust_aust_rate", aust_exchange_rate.to_string());
    }*/
    // pool_info.luna_cap ? instead of tax_cap
    if (requested_asset.amount + tax_buffer) > luna_available {
        // If we need too, withdraw from the various passive strategies, initially defined as the bLuna-Luna LP. This method will return both assets and assumes no desired assets as-is
        // TODO: Add a flag to this method so that a user can specify if they want luna or bluna, we can expand this later with an enum to offer a quick way to get any variant of luna via swapp ;-)
        // TODO: NOTE: Check the clone usage, added it to fixup tests
        let _ = withdraw_passive_strategy(
            &deps.as_ref(),
            requested_asset.amount,
            state.bluna_address,
            &get_lp_token_address(&deps.as_ref(), state.astro_lp_address.clone())?,
            &state.astro_lp_address,
            response.clone(),
        )?;
    }

    // If caller not whitelisted, calculate flashloan fee

    // let loan_fee: Uint128 = if whitelisted {
    //     Uint128::zero()
    // } else {
    //     fees.flash_loan_fee.compute(requested_asset.amount)
    // };
    //TODO why charge everybody? in theory we don't want to charge fees for ourselves, but to others.
    // for now the vault will be closed to the public but i guess we can keep the logic in place.
    // Either that or remove the fee completely and then reintroduce it if we want to open the vault
    let loan_fee = fees.flash_loan_fee.compute(requested_asset.amount);

    // NOTE: Forget the whitelist and just charge everyone, why should anyone get free flashloans ?
    fees.flash_loan_fee.compute(requested_asset.amount);
    // Construct transfer of funds msg, tax is accounted for by buffer
    let loan_msg = into_msg_without_tax(requested_asset, info.sender.clone())?;
    response = response.add_message(loan_msg);

    // Construct return call with received binary
    let return_call = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.sender.into(),
        msg: payload.callback,
        funds: vec![],
    });

    response = response.add_message(return_call);

    // Sets the current value of the vault and save logs
    response = response.add_attributes(before_trade(deps.branch(), env.clone())?);

    // Call encapsulate function
    encapsulate_payload(deps.as_ref(), env, response, loan_fee)
}

/// Resets last trade and sets current UST balance of caller
pub fn before_trade(deps: DepsMut, env: Env) -> Result<Vec<(&str, String)>, LunaVaultError> {
    let mut profit_check = PROFIT.load(deps.storage)?;

    // last_balance call can not be reset until after the loan.
    if profit_check.last_balance != Uint128::zero() {
        return Err(LunaVaultError::Nonzero {});
    }

    profit_check.last_profit = Uint128::zero();

    // Index 0 = total_value
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    profit_check.last_balance = compute_total_value(&env, deps.as_ref(), &info)?.0;
    PROFIT.save(deps.storage, &profit_check)?;

    Ok(vec![(
        "value before trade: ",
        profit_check.last_balance.to_string(),
    )])
}

/// Checks if balance increased after the trade
pub fn after_trade(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    loan_fee: Uint128,
) -> VaultResult<Response> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (_, luna_in_contract, _, _, _) = compute_total_value(&env, deps.as_ref(), &info)?;
    let state = STATE.load(deps.storage)?;
    // Deposit funds into a passive strategy again if applicable.
    let mut response = Response::default();
    //TODO revise what is it that we are depositing here into the passive strategy.
    // After trade prob should check if we did the flashloan with bluna,cluna or just pure luna.
    // if the flashloan returned luna, then somehow account for the liquid luna that was sitting there waiting to
    // be withdrawn
    response = deposit_passive_strategy(
        &deps.as_ref(),
        luna_in_contract,
        state.bluna_address.clone(),
        &state.astro_lp_address,
        response.clone(),
    )?;

    let mut conf = PROFIT.load(deps.storage)?;

    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let balance = compute_total_value(&env, deps.as_ref(), &info)?.0;

    // Check if balance increased with expected fee, otherwise cancel everything
    if balance < conf.last_balance + loan_fee {
        return Err(LunaVaultError::CancelLosingTrade {});
    }

    let profit = balance - conf.last_balance;

    conf.last_profit = profit;
    conf.last_balance = Uint128::zero();
    PROFIT.save(deps.storage, &conf)?;

    let commission_response = send_commissions(deps.as_ref(), msg_info, profit)?;
    response = response
        // Send commission of profit to Treasury
        .add_submessages(commission_response.messages)
        .add_attributes(commission_response.attributes)
        .add_attribute("value after commission: ", balance.to_string());

    //TODO again deposit to passive strategy??
    deposit_passive_strategy(
        &deps.as_ref(),
        luna_in_contract,
        state.bluna_address,
        &state.astro_lp_address,
        response,
    )
}

///TODO potentially improve this function by passing the Asset, so that this component could be reused for other vaults
/// Sends the commission fee which is a function of the profit made by the contract, forwarded by the profit-check contract
fn send_commissions(deps: Deps, _info: MessageInfo, profit: Uint128) -> VaultResult<Response> {
    let fees = FEE.load(deps.storage)?;

    let commission_amount = fees.commission_fee.compute(profit);

    // Construct commission msg
    let refund_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: LUNA_DENOM.to_string(),
        },
        amount: commission_amount,
    };
    let commission_msg = refund_asset.into_msg(&deps.querier, fees.treasury_addr)?;

    Ok(Response::new()
        .add_attribute("treasury commission:", commission_amount.to_string())
        .add_message(commission_msg))
}

/// Helper method which encapsulates the requested funds.
/// This function prevents callers from doing unprofitable actions
/// with the vault funds and makes sure the funds are returned by
/// the borrower.
pub fn encapsulate_payload(
    _deps: Deps,
    env: Env,
    response: Response,
    loan_fee: Uint128,
) -> VaultResult<Response> {
    let total_response: Response = Response::new().add_attributes(response.attributes);

    // Callback for after the loan
    let after_trade = CallbackMsg::AfterTrade { loan_fee }.to_cosmos_msg(&env.contract.address)?;

    Ok(total_response
        // Add response that:
        // 1. Withdraws funds from Passive Strategy if needed
        // 2. Sends funds to the borrower
        // 3. Calls the borrow contract through the provided callback msg
        .add_submessages(response.messages)
        // After borrower actions, deposit the received funds back into
        // Passive Strategy if applicable
        // Call profit-check to cancel the borrow if
        // no profit is made.
        .add_message(after_trade))
}

/// Handles the callback after using a flashloan
pub fn _handle_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CallbackMsg,
) -> VaultResult<Response> {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(LunaVaultError::NotCallback {});
    }
    match msg {
        CallbackMsg::AfterTrade { loan_fee } => after_trade(deps, env, info, loan_fee),
    }
}
