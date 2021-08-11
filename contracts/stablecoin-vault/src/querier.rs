use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::STATE;

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary,  Coin, Decimal, Deps,
    QueryRequest, StdResult, Uint128, WasmQuery,
};
use terra_cosmwasm::TerraQuerier;

pub fn from_micro(
    amount: Uint128
) -> Decimal {
    Decimal::from_ratio(amount, Uint128::from(1000000u64))
}

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
    deps: Deps
) -> StdResult<EpochStateResponse> {
    let state = STATE.load(deps.storage)?;
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr:state.anchor_money_market_address.to_string(),
        msg: to_binary(&AnchorQuery::EpochState {
            block_height: None,
            distributed_interest: None,
        })?,
    }))
}

pub fn query_market_price(
    deps: Deps,
    offer_coin: Coin,
    ask_denom: String
) -> StdResult<Uint128> {
    let querier = TerraQuerier::new(&deps.querier);
    let response = querier.query_swap(offer_coin, ask_denom)?;
    Ok(response.receive.amount)
}
