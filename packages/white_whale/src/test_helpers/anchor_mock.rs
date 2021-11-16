use cosmwasm_std::{from_binary, to_binary, Binary, Empty, Response, StdResult, Uint128};
use cosmwasm_bignumber::{Decimal256, Uint256};

use cw20::Cw20ReceiveMsg;
use cw_multi_test::{Contract, ContractWrapper};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{AssetInfo, Asset};
use crate::query::anchor::{AnchorQuery, EpochStateResponse};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MockInstantiateMsg {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PingMsg {
    pub payload: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MockExecuteMsg {
    Receive(Cw20ReceiveMsg),
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 2],
    pub total_share: Uint128,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairResponse {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: String,
    pub liquidity_token: String
}


pub fn contract_anchor_mock() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |_, _, _, msg: MockExecuteMsg| -> StdResult<Response> {
            match msg {
                MockExecuteMsg::Receive(Cw20ReceiveMsg {
                    sender: _,
                    amount: _,
                    msg,
                }) => {
                    let received: PingMsg = from_binary(&msg)?;
                    Ok(Response::new()
                        .add_attribute("action", "pong")
                        .set_data(to_binary(&received.payload)?))
                }
            }
        },
        |_, _, _, _: MockInstantiateMsg| -> StdResult<Response> { Ok(Response::default()) },
        |_, _, msg: AnchorQuery| -> StdResult<Binary> {  match msg {
                AnchorQuery::EpochState {
                    distributed_interest,
                    block_height
                } => Ok(to_binary(&mock_epoch_state())?),
        }},
    );
    Box::new(contract)
}



pub fn mock_epoch_state() -> EpochStateResponse {
    println!("Made it to mock");

    // println!("{:?}", distributed_interest);
    // println!("{:?}", block_height);
    let epoch_state: EpochStateResponse = EpochStateResponse{
        exchange_rate: Decimal256::percent(120),
        aterra_supply: Uint256::from(1000000u64)
    };
    return epoch_state;
}
