use std::borrow::BorrowMut;

use cosmwasm_std::{Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::Asset;
use terraswap::querier::query_supply;

use white_whale::anchor::anchor_deposit_msg;
use white_whale::astroport_helper::{create_astroport_lp_msg, create_astroport_msg};

use crate::contract::compute_total_value;
use crate::error::LunaVaultError;
use crate::helpers::slashing;
use crate::math::decimal_division;
use crate::pool_info::PoolInfoRaw;
use crate::queries::query_total_lp_issued;
use crate::state::{CURRENT_BATCH, DEPOSIT_INFO, PARAMETERS, POOL_INFO, PROFIT, STATE};

// Deposits Luna into the contract.
pub fn provide_liquidity(
    mut deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    asset: Asset,
) -> VaultResult {
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    let profit = PROFIT.load(deps.storage)?;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;

    if profit.last_balance != Uint128::zero() {
        return Err(LunaVaultError::DepositDuringLoan {});
    }

    // Init vector for logging
    let mut attrs = vec![];
    // Check if deposit matches claimed deposit.
    deposit_info.assert(&asset.info)?;
    asset.assert_sent_native_token_balance(&msg_info)?;
    attrs.push(("Action:", String::from("Deposit to vault")));
    attrs.push(("Received funds:", asset.to_string()));

    let params = PARAMETERS.load(deps.storage)?;
    let threshold = params.er_threshold;
    let recovery_fee = params.peg_recovery_fee;

    // current batch requested fee is needed for accurate exchange rate computation.
    let current_batch = CURRENT_BATCH.load(deps.storage)?;
    let requested_with_fee = current_batch.requested_with_fee;

    // Received deposit to vault
    let deposit: Uint128 = asset.amount;

    // check slashing
    let mut state = STATE.load(deps.storage)?;
    slashing(&mut deps, env.clone(), &mut state, &params)?;

    // get the total vluna supply
    let mut total_supply = query_supply(&deps.querier, info.liquidity_token.clone())?;

    // peg recovery fee should be considered
    let mint_amount = decimal_division(deposit, state.exchange_rate);
    let mut mint_amount_with_fee = mint_amount;
    if state.exchange_rate < threshold {
        let max_peg_fee = mint_amount * recovery_fee;
        let required_peg_fee = ((total_supply + mint_amount + current_batch.requested_with_fee)
            .checked_sub(state.total_bond_amount + deposit))?;
        let peg_fee = Uint128::min(max_peg_fee, required_peg_fee);
        mint_amount_with_fee = (mint_amount.checked_sub(peg_fee))?;
    }

    // total supply should be updated for exchange rate calculation.
    total_supply += mint_amount_with_fee;

    // exchange rate should be updated for future
    state.total_bond_amount += deposit;
    state.update_exchange_rate(total_supply, requested_with_fee);
    STATE.save(deps.storage, &state)?;

    // mint LP token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: msg_info.sender.to_string(),
            amount: mint_amount_with_fee,
        })?,
        funds: vec![],
    });

    let response = Response::new().add_attributes(attrs).add_message(msg);
    // If contract holds more than ASTROPORT_DEPOSIT_THRESHOLD [LUNA] then try deposit to Astroport and leave LUNA_CAP [LUNA] in contract.
    let (_, luna_in_contract, _, _, _) = compute_total_value(&env, deps.as_ref(), &info)?;
    return if luna_in_contract > info.luna_cap {
        _deposit_passive_strategy(response)?;
    } else {
        Ok(response)
    };
}

// Deposits Luna into the passive strategy (Astroport) -> luna-bluna LP
fn _deposit_passive_strategy(response: Response) -> VaultResult {
    let deposit_msg = create_astroport_lp_msg()?;

    Ok(response.add_message(deposit_msg))
}
