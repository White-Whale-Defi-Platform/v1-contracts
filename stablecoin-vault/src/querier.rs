use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{QueryMsg as PairQueryMsg, SimulationResponse};
use crate::state::{ANCHOR, LUNA_DENOM};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Api,  Coin, Decimal,
    Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, Uint128, WasmQuery,
};
use terra_cosmwasm::TerraQuerier;

pub fn from_micro(
    amount: Uint128
) -> Decimal {
    Decimal::from_ratio(amount, Uint128(1000000))
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

pub fn query_aust_exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<EpochStateResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: HumanAddr::from(ANCHOR),
        msg: to_binary(&AnchorQuery::EpochState {
            block_height: None,
            distributed_interest: None,
        })?,
    }))
}

pub fn query_luna_price_on_terraswap<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pool_address: HumanAddr,
) -> StdResult<Uint128> {
    let response: SimulationResponse = deps.querier.query(
        &QueryRequest::Wasm(WasmQuery::Smart{
            contract_addr: pool_address,
            msg: to_binary(&PairQueryMsg::Simulation{
                offer_asset: Asset{
                    info: AssetInfo::NativeToken{ denom: LUNA_DENOM.to_string() },
                    amount: Uint128(1000000),
                }
            })?
        })
    )?;

    Ok(response.return_amount)
}


pub fn query_luna_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    ask_denom: String
) -> StdResult<Uint128> {
    let querier = TerraQuerier::new(&deps.querier);
    let response = querier.query_swap(Coin{ denom: "uluna".to_string(), amount: Uint128(1000000) }, ask_denom)?;
    Ok(response.receive.amount)
}
