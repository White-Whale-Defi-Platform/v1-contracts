use crate::contract::VaultResult;
use crate::helpers::{compute_total_value, get_withdraw_fee};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::state::{State, DEPOSIT_INFO, FEE, POOL_INFO, PROFIT, STATE};
use cosmwasm_std::{Coin, Deps, Env, Uint128};
use terraswap::asset::Asset;
use terraswap::querier::query_supply;
use white_whale::luna_vault::msg::{
    EstimateWithdrawFeeResponse, FeeResponse, LastBalanceResponse, LastProfitResponse,
    PoolResponse, ValueResponse,
};

/// Queries the PoolInfo configuration
pub fn query_pool_info(deps: Deps) -> VaultResult<PoolInfo> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    info.to_normal(deps)
}

/// Queries pool state
pub fn try_query_pool_state(env: Env, deps: Deps) -> VaultResult<PoolResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let assets: [Asset; 4] = info.query_pools(deps, info.contract_addr.clone())?;
    let total_share: Uint128 = query_supply(&deps.querier, info.liquidity_token.clone())?;

    let (total_value_in_luna, _, _, _, _) = compute_total_value(&env, deps, &info)?;

    Ok(PoolResponse {
        assets,
        total_value_in_luna,
        total_share,
        liquidity_token: info.liquidity_token.into(),
    })
}

/// Queries contract [State]
pub fn query_state(deps: Deps) -> VaultResult<State> {
    Ok(STATE.load(deps.storage)?)
}

/// Queries Fees
pub fn query_fees(deps: Deps) -> VaultResult<FeeResponse> {
    Ok(FeeResponse {
        fees: FEE.load(deps.storage)?,
    })
}

/// Queries total luna value in vault
pub fn query_total_value(env: Env, deps: Deps) -> VaultResult<ValueResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (total_luna_value, _, _, _, _) = compute_total_value(&env, deps, &info)?;
    Ok(ValueResponse { total_luna_value })
}

/// Queries estimated withdrawal fee
pub fn estimate_withdraw_fee(
    deps: Deps,
    amount: Uint128,
) -> VaultResult<EstimateWithdrawFeeResponse> {
    let fee = get_withdraw_fee(deps, amount)?;
    Ok(EstimateWithdrawFeeResponse {
        fee: vec![Coin {
            denom: DEPOSIT_INFO.load(deps.storage)?.get_denom()?,
            amount: fee,
        }],
    })
}

/// Queries vault's last profit
pub fn query_last_profit(deps: Deps) -> VaultResult<LastProfitResponse> {
    let conf = PROFIT.load(deps.storage)?;
    Ok(LastProfitResponse {
        last_profit: conf.last_profit,
    })
}

/// Queries vault's last balance
pub fn query_last_balance(deps: Deps) -> VaultResult<LastBalanceResponse> {
    let conf = PROFIT.load(deps.storage)?;
    Ok(LastBalanceResponse {
        last_balance: conf.last_balance,
    })
}
