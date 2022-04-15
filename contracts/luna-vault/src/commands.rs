use cosmwasm_std::{Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::Asset;
use terraswap::querier::query_supply;
use white_whale::anchor::anchor_deposit_msg;
use white_whale::astroport_helper::{create_astroport_lp_msg, create_astroport_msg};

use crate::contract::compute_total_value;
use crate::error::LunaVaultError;
use crate::pool_info::PoolInfoRaw;
use crate::state::{DEPOSIT_INFO, PARAMETERS, POOL_INFO, PROFIT, STATE};

// Deposits Luna into the contract.
pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    asset: Asset,
) -> VaultResult {
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    let profit = PROFIT.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let denom = deposit_info.clone().get_denom()?;
    let params = PARAMETERS.load(deps.storage)?;
    let threshold = params.er_threshold;
    let recovery_fee = params.peg_recovery_fee;

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

    // Received deposit to vault
    let deposit: Uint128 = asset.amount;

    ///TODO double check this logic
    // Get total value in Vault
    let (total_deposits_in_luna, luna_in_contract, _) =
        compute_total_value(&env, deps.as_ref(), &info)?;
    // Get total supply of LP tokens and calculate share
    let total_share = query_supply(&deps.querier, info.liquidity_token.clone())?;

    let share = if total_share == Uint128::zero()
        || total_deposits_in_luna.checked_sub(deposit)? == Uint128::zero()
    {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_luna.checked_sub(deposit)?)
    };

    // mint LP token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: msg_info.sender.to_string(),
            amount: share,
        })?,
        funds: vec![],
    });

    let response = Response::new().add_attributes(attrs).add_message(msg);
    // If contract holds more than ASTROPORT_DEPOSIT_THRESHOLD [LUNA] then try deposit to Astroport and leave LUNA_CAP [LUNA] in contract.
    return if luna_in_contract > info.luna_cap * Decimal::percent(150) {
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
