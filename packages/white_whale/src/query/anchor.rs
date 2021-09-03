use cosmwasm_std::{ to_binary, Deps, QueryRequest, StdResult, WasmQuery };
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_bignumber::{Decimal256, Uint256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorQuery {
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}


pub fn query_aust_exchange_rate(
    deps: Deps,
    anchor_money_market_address: String
) -> StdResult<EpochStateResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: anchor_money_market_address,
        msg: to_binary(&AnchorQuery::EpochState {
            block_height: None,
            distributed_interest: None
        })?,
    }))
}