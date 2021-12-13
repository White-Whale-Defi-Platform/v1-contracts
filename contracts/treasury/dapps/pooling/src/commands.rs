use std::borrow::BorrowMut;

use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal, Deps, Env, Fraction, MessageInfo, Response, Uint128,
    WasmMsg, DepsMut, from_binary,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::asset::Asset;
use terraswap::pair::{Cw20HookMsg, PoolResponse};

use terraswap::querier::query_supply;
use white_whale::fee::Fee;
use white_whale::query::terraswap::{query_asset_balance, query_pool};
use white_whale::treasury::dapp_base::common::PAIR_POSTFIX;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use crate::contract::PoolingResult;
use crate::error::PoolingError;
use crate::state::{State, POOL, Pool, FEE, STATE};

use white_whale::treasury::msg::send_to_treasury;

/// handler function invoked when the stablecoin-vault contract receives
/// a transaction. In this case it is triggered when the LP tokens are deposited
/// into the contract
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> PoolingResult {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Swap { .. } => Err(PoolingError::NoSwapAvailable {}),
        Cw20HookMsg::WithdrawLiquidity {} => {
            let state: State = STATE.load(deps.storage)?;
            if msg_info.sender != state.lp_token_addr {
                return Err(PoolingError::NotLPToken { token: msg_info.sender.to_string()});
            }
            try_withdraw_liquidity(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
    }
}


pub fn try_provide_liquidity(deps: DepsMut, msg_info: MessageInfo, asset: Asset) -> PoolingResult {
    let pool: Pool = POOL.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    
    // Init vector for logging
    let mut attrs = vec![];
    // Check if deposit matches claimed deposit.
    .assert(&asset.info)?;
    asset.assert_sent_native_token_balance(&msg_info)?;
    attrs.push(("Action:", String::from("Deposit to vault")));
    attrs.push(("Received funds:", asset.to_string()));

    // Received deposit to vault
    let deposit: Uint128 = asset.amount;

    // Get total value in Vault
    let (total_deposits_in_ust, stables_in_contract, _) =
        compute_total_value(deps.as_ref(), &info)?;
    // Get total supply of LP tokens and calculate share
    let total_share = query_supply(
        &deps.querier,
        deps.api.addr_humanize(&info.liquidity_token)?,
    )?;

    let share = if total_share == Uint128::zero() {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_ust - deposit)
    };


    // mint LP token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: msg_info.sender.to_string(),
            amount: share,
        })?,
        funds: vec![],
    });

    let response = Response::new().add_attributes(attrs).add_message(msg);

    // If contract holds more then ANCHOR_DEPOSIT_THRESHOLD [UST] then try deposit to anchor and leave UST_CAP [UST] in contract.
    if stables_in_contract > info.stable_cap * Decimal::percent(150) {

        let deposit_amount = stables_in_contract - info.stable_cap;
        let anchor_deposit = Coin::new(deposit_amount.u128(), denom);
        let deposit_msg = anchor_deposit_msg(
            deps.as_ref(),
            deps.api.addr_humanize(&state.anchor_money_market_address)?,
            anchor_deposit,
        )?;
        return Ok(response.add_message(deposit_msg));
    };

    Ok(response)
}

/// Attempt to withdraw deposits. Fees are calculated and deducted in lp tokens.
/// This allowes the war-chest to accumulate a stake in the vault.
/// The refund is taken out of Anchor if possible.
/// Luna holdings are not eligible for withdrawal.
pub fn try_withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    sender: String,
    amount: Uint128,
) -> PoolingResult {
    let pool: Pool = POOL.load(deps.storage)?;
    let state: State = STATE.load(deps.storage)?;
    let fee: Fee = FEE.load(deps.storage)?;

    // Logging var
    let mut attrs = vec![];

    // Calculate share of pool and requested pool value
    let total_share: Uint128 = query_supply(&deps.querier, state.lp_token_addr)?;
    // Get treasury fee in LP tokens
    let treasury_fee = fee.share * amount;
    // Share with fee deducted.
    let share_ratio: Decimal = Decimal::from_ratio(amount - treasury_fee, total_share);

    // Init response
    let mut response = Response::new();
    // Available aUST
    

    // Construct warchest fee msg.
    let warchest_fee_msg = fee_config.warchest_fee.msg(
        deps.as_ref(),
        lp_token_warchest_fee,
        deps.api.addr_humanize(&fee_config.warchest_addr)?,
    )?;
    attrs.push(("War chest fee:", warchest_fee.to_string()));

    // Construct refund message
    let refund_asset = Asset {
        info: AssetInfo::NativeToken { denom },
        amount: refund_amount,
    };
    let tax_assed = refund_asset.deduct_tax(&deps.querier)?;

    let refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: sender,
        amount: vec![tax_assed],
    });
    // LP burn msg
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        // Burn exludes fee
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: (amount - warchest_fee),
        })?,
        funds: vec![],
    });


    Ok(response
        .add_message(refund_msg)
        .add_message(burn_msg)
        .add_message(warchest_fee_msg)
        .add_attribute("action:", "withdraw_liquidity")
        .add_attributes(attrs))
}
