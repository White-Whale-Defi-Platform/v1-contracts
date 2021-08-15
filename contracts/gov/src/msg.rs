use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Decimal,Uint128};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    EndPoll {
        poll_id: u64,
    },
    ExecutePoll {
        poll_id: u64,
    },
    RegisterContracts {
        whale_token: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Config returns the configuration values of the governance contract
    Config {},
    // State returns the governance state values such as the poll_count and the amount deposited
    State {},
    // Poll returns the information related to a Poll if that poll exists 
    Poll {
        poll_id: u64,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}
