use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, Env, QueryRequest, StdResult, Uint128, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorQuery {
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
    UnbondRequests {
        address: String,
    },
    WithdrawableUnbonded {
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondRequestsResponse {
    pub address: String,
    pub requests: UnbondRequest,
}

pub type UnbondRequest = Vec<(u64, Uint128, Uint128)>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint128,
}

pub fn query_aust_exchange_rate(
    env: Env,
    deps: Deps,
    anchor_money_market_address: String,
) -> StdResult<Decimal> {
    let response: EpochStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: anchor_money_market_address,
            msg: to_binary(&AnchorQuery::EpochState {
                block_height: Some(env.block.height),
                distributed_interest: None,
            })?,
        }))?;
    Ok(Decimal::from(response.exchange_rate))
}

pub fn query_unbond_requests(
    deps: Deps,
    bluna_hub_address: Addr,
    address: Addr,
) -> StdResult<UnbondRequestsResponse> {
    let response: UnbondRequestsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: bluna_hub_address.to_string(),
            msg: to_binary(&AnchorQuery::UnbondRequests {
                address: address.to_string(),
            })?,
        }))?;

    Ok(response)
}

pub fn query_withdrawable_unbonded(
    deps: Deps,
    bluna_hub_address: Addr,
    address: Addr,
) -> StdResult<WithdrawableUnbondedResponse> {
    let response: WithdrawableUnbondedResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: bluna_hub_address.to_string(),
            msg: to_binary(&AnchorQuery::WithdrawableUnbonded {
                address: address.to_string(),
            })?,
        }))?;

    Ok(response)
}
