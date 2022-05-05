use cosmwasm_std::{Coin, Deps, Env, Storage, Uint128};
use terraswap::asset::Asset;
use terraswap::querier::query_supply;

use white_whale::luna_vault::msg::{
    EstimateWithdrawFeeResponse, FeeResponse, LastBalanceResponse, LastProfitResponse,
    PoolResponse, ValueResponse,
};
use white_whale::memory::queries::query_contract_from_mem;
use white_whale::memory::ANCHOR_BLUNA_HUB_ID;
use white_whale::query::anchor::{UnbondRequestsResponse, WithdrawableUnbondedResponse};

use crate::contract::VaultResult;
use crate::error::LunaVaultError;
use crate::helpers::{compute_total_value, get_withdraw_fee};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::state::{
    State, DEFAULT_UNBOND_EXPIRATION_TIME, DEPOSIT_INFO, FEE, POOL_INFO, PROFIT, STATE,
    UNBOND_HANDLERS_ASSIGNED, UNBOND_HANDLER_EXPIRATION_TIME,
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

/// Queries withdrawable unbonded amount for the unbond handler associated with the given address
pub fn query_withdrawable_unbonded(
    deps: Deps,
    address: String,
) -> VaultResult<WithdrawableUnbondedResponse> {
    let address = deps.api.addr_validate(&address)?;
    let unbond_handler = UNBOND_HANDLERS_ASSIGNED
        .may_load(deps.storage, address)?
        .ok_or(LunaVaultError::NoUnbondHandlerAssigned {})?;

    let state = STATE.load(deps.storage)?;
    let bluna_hub_address =
        query_contract_from_mem(deps, &state.memory_address, ANCHOR_BLUNA_HUB_ID)?;

    // query how much withdrawable_unbonded is on anchor for the given unbond handler
    Ok(white_whale::query::anchor::query_withdrawable_unbonded(
        deps,
        bluna_hub_address,
        unbond_handler,
    )?)
}

/// Queries unbond requests for the unbond handler associated with the given address
pub fn query_unbond_requests(deps: Deps, address: String) -> VaultResult<UnbondRequestsResponse> {
    let address = deps.api.addr_validate(&address)?;
    let unbond_handler = UNBOND_HANDLERS_ASSIGNED
        .may_load(deps.storage, address)?
        .ok_or(LunaVaultError::NoUnbondHandlerAssigned {})?;

    let state = STATE.load(deps.storage)?;
    let bluna_hub_address =
        query_contract_from_mem(deps, &state.memory_address, ANCHOR_BLUNA_HUB_ID)?;

    // query unbond requests on anchor for the given unbond handler
    Ok(white_whale::query::anchor::query_unbond_requests(
        deps,
        bluna_hub_address,
        unbond_handler,
    )?)
}

/// Queries the unbond handler expiration time if set, returns the default value otherwise
pub fn query_unbond_handler_expiration_time(storage: &dyn Storage) -> VaultResult<u64> {
    let expiration_time = UNBOND_HANDLER_EXPIRATION_TIME.may_load(storage)?;

    if let Some(expiration_time) = expiration_time {
        Ok(expiration_time)
    } else {
        Ok(DEFAULT_UNBOND_EXPIRATION_TIME)
    }
}
