use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CosmosMsg, Empty, Uint128};

use crate::vault_assets::VaultAsset;
use terraswap::asset::AssetInfo;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit {},
    Spend {
        recipient: String,
        amount: Uint128,
    },
    TraderAction {
        target: String,
        msgs: Vec<CosmosMsg<Empty>>,
    },
    AddTrader {
        trader: String,
    },
    UpdateAssets {
        to_add: Vec<VaultAsset>,
        to_remove: Vec<AssetInfo>,
    },
    RemoveTrader {
        trader: String,
    },
    UpdateBalances {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetTotalValue {},
    GetHoldingValue { identifier: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub traders: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalValueResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoldingValueResponse {
    pub value: Uint128,
}
