use cosmwasm_std::{to_binary, Addr, Deps, QueryRequest, StdResult, Uint128, WasmQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrismQuery {
    UnbondRequests { address: String },
    WithdrawableUnbonded { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondRequestsResponse {
    pub address: String,
    pub requests: UnbondRequest,
}

pub type UnbondRequest = Vec<(u64, Uint128)>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint128,
}

pub fn query_unbond_requests(
    deps: Deps,
    cluna_hub_address: Addr,
    address: Addr,
) -> StdResult<UnbondRequestsResponse> {
    let response: UnbondRequestsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cluna_hub_address.to_string(),
            msg: to_binary(&PrismQuery::UnbondRequests {
                address: address.to_string(),
            })?,
        }))?;

    Ok(response)
}

pub fn query_withdrawable_unbonded(
    deps: Deps,
    cluna_hub_address: Addr,
    address: Addr,
) -> StdResult<WithdrawableUnbondedResponse> {
    let response: WithdrawableUnbondedResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cluna_hub_address.to_string(),
            msg: to_binary(&PrismQuery::WithdrawableUnbonded {
                address: address.to_string(),
            })?,
        }))?;

    Ok(response)
}
