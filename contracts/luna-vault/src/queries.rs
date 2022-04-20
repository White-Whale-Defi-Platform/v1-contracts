use cosmwasm_std::{Coin, Deps, Env, StdResult, Uint128};
use terraswap::asset::Asset;
use terraswap::querier::query_supply;
use white_whale::luna_vault::msg::{AllHistoryResponse, CurrentBatchResponse, EstimateWithdrawFeeResponse, FeeResponse, LastBalanceResponse, LastProfitResponse, PoolResponse, UnbondRequestsResponse, ValueResponse, WithdrawableUnbondedResponse};
use crate::helpers::{compute_total_value, get_withdraw_fee};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::state::{all_unbond_history, CURRENT_BATCH, DEPOSIT_INFO, FEE, get_unbond_requests, PARAMETERS, Parameters, POOL_INFO, PROFIT, query_get_finished_amount, STATE, State, UNBOND_WAITLIST};

/// Queries the PoolInfo configuration
pub fn query_pool_info(deps: Deps) -> StdResult<PoolInfo> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    info.to_normal(deps)
}

/// Queries pool state
pub fn try_query_pool_state(env: Env, deps: Deps) -> StdResult<PoolResponse> {
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
pub fn query_state(deps: Deps) -> StdResult<State> {
    STATE.load(deps.storage)
}

/// Queries Fees
pub fn query_fees(deps: Deps) -> StdResult<FeeResponse> {
    Ok(FeeResponse {
        fees: FEE.load(deps.storage)?,
    })
}

/// Queries total luna value in vault
pub fn query_total_value(env: Env, deps: Deps) -> StdResult<ValueResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (total_luna_value, _, _, _, _) = compute_total_value(&env, deps, &info)?;
    Ok(ValueResponse { total_luna_value })
}

/// Queries estimated withdrawal fee
pub fn estimate_withdraw_fee(
    deps: Deps,
    amount: Uint128,
) -> StdResult<EstimateWithdrawFeeResponse> {
    let fee = get_withdraw_fee(deps, amount)?;
    Ok(EstimateWithdrawFeeResponse {
        fee: vec![Coin {
            denom: DEPOSIT_INFO.load(deps.storage)?.get_denom()?,
            amount: fee,
        }],
    })
}

/// Queries vault's last profit
pub fn query_last_profit(deps: Deps) -> StdResult<LastProfitResponse> {
    let conf = PROFIT.load(deps.storage)?;
    Ok(LastProfitResponse {
        last_profit: conf.last_profit,
    })
}

/// Queries vault's last balance
pub fn query_last_balance(deps: Deps) -> StdResult<LastBalanceResponse> {
    let conf = PROFIT.load(deps.storage)?;
    Ok(LastBalanceResponse {
        last_balance: conf.last_balance,
    })
}

/// Queries withdrawable unbonded
pub fn query_withdrawable_unbonded(
    deps: Deps,
    address: String,
    env: Env,
) -> StdResult<WithdrawableUnbondedResponse> {
    let params = PARAMETERS.load(deps.storage)?;
    let historical_time = env.block.time.seconds() - params.unbonding_period;
    let addr = deps.api.addr_validate(&address)?;
    // query the finished amount with the default limit (None), to obtain the same value as the result of the actual unbond operation
    let all_requests = query_get_finished_amount(deps.storage, &addr, historical_time, None)?;

    let withdrawable = WithdrawableUnbondedResponse {
        withdrawable: all_requests,
    };
    Ok(withdrawable)
}

pub fn query_current_batch(deps: Deps) -> StdResult<CurrentBatchResponse> {
    let current_batch = CURRENT_BATCH.load(deps.storage)?;
    Ok(CurrentBatchResponse {
        id: current_batch.id,
        requested_with_fee: current_batch.requested_with_fee,
    })
}

pub fn query_params(deps: Deps) -> StdResult<Parameters> {
    PARAMETERS.load(deps.storage)
}

pub fn query_unbond_requests(
    deps: Deps,
    address: String,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<UnbondRequestsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let requests = get_unbond_requests(deps.storage, &addr, start, limit)?;
    let res = UnbondRequestsResponse { address, requests };
    Ok(res)
}

pub fn query_unbond_requests_limitation(
    deps: Deps,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<AllHistoryResponse> {
    let requests = all_unbond_history(deps.storage, start, limit)?;
    // TODO: fixup
    // let requests_res = requests.iter().map(|item| item.into()).collect();
    let res = AllHistoryResponse {
        history: vec![],
    };
    Ok(res)
}
