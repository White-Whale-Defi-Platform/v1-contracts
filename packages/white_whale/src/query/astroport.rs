use cosmwasm_std::{
    to_binary, Addr, AllBalanceResponse,BalanceResponse,BankQuery, Coin,QuerierWrapper, Decimal, Deps, QueryRequest, StdResult, Uint128, WasmQuery,
};
// use terraswap::asset::{Asset, AssetInfo, PairInfo};
// use terraswap::pair::{PoolResponse, QueryMsg, SimulationResponse};
use crate::astroport_helper::*;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};





pub fn simulate_swap(deps: Deps, pool_address: Addr, offer_coin: Coin) -> StdResult<Uint128> {
    let response: SimulationResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_address.to_string(),
            msg: to_binary(&QueryMsg::Simulation {
                offer_asset: Asset {
                    info: AssetInfo::NativeToken {
                        denom: offer_coin.denom,
                    },
                    amount: offer_coin.amount,
                },
            })?,
        }))?;

    Ok(response.return_amount)
}

// perform a query for Pool information using the provided pool_address
// return any response.
// PoolResponse comes from terraswap and contains info on each of the assets as well as total share
pub fn query_pool(deps: Deps, pool_address: Addr) -> StdResult<PoolResponse> {
    let response: PoolResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pool {})?,
    }))?;

    Ok(response)
}

// perform a query for the LP Token Pair information using the provided pool_address
// return only the address. TODO: Review if we should return the full response instead
pub fn query_lp_token(deps: Deps, pool_address: Addr) -> StdResult<String> {
    let response: PairInfo = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pair {})?,
    }))?;

    Ok(response.liquidity_token.to_string())
}

pub fn pool_ratio(deps: Deps, pool_address: Addr) -> StdResult<Decimal> {
    let response: PoolResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pool {})?,
    }))?;
    // [ust,luna]
    let ratio = Decimal::from_ratio(response.assets[0].amount, response.assets[1].amount);
    Ok(ratio)
}

pub fn query_asset_balance(
    deps: Deps,
    asset_info: &AssetInfo,
    address: Addr,
) -> StdResult<Uint128> {
    let amount = match asset_info.clone() {
        AssetInfo::NativeToken { denom } => query_balance(&deps.querier, address, denom)?,
        AssetInfo::Token { contract_addr } => query_token_balance(
            &deps.querier,
            deps.api.addr_validate(contract_addr.as_str())?,
            address,
        )?,
    };
    Ok(amount)
}


//from https://github.com/astroport-fi/astroport/blob/master/packages/astroport/src/querier.rs
pub fn query_balance(
    querier: &QuerierWrapper,
    account_addr: Addr,
    denom: String,
) -> StdResult<Uint128> {
    let balance: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: String::from(account_addr),
        denom,
    }))?;
    Ok(balance.amount.amount)
}

pub fn query_token_balance(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    let res: Cw20BalanceResponse = querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: String::from(account_addr),
            })?,
        }))
        .unwrap_or_else(|_| Cw20BalanceResponse {
            balance: Uint128::zero(),
        });

    Ok(res.balance)
}