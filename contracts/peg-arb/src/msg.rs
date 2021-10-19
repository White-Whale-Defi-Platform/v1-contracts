use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Decimal, StdResult, WasmMsg};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub trader: String,
    pub vault_address: String,
    pub seignorage_address: String,
    pub pool_address: String,
    pub asset_info: AssetInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    BorrowForArb {
        details: ArbDetails,
        above_peg: bool,
    },
    AbovePegCallback {
        details: ArbDetails,
    },
    BelowPegCallback {
        details: ArbDetails,
    },
    SendToVault {},
    TestMsg {},
    SetAdmin {
        admin: String,
    },
    SetTrader {
        trader: String,
    },
    Callback(CallbackMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    AfterSuccessfulTradeCallback {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ArbDetails {
    pub coin: Coin,
    pub slippage: Decimal,
    pub belief_price: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}
