use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, CosmosMsg, Empty, StdResult, Uint128, WasmMsg};

use crate::treasury::vault_assets::VaultAsset;
use terraswap::asset::AssetInfo;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Sets the admin
    SetAdmin {
        admin: String,
    },
    /// Executes the provided messages if sender is whitelisted
    TraderAction {
        msgs: Vec<CosmosMsg<Empty>>,
    },
    /// Adds the provided address to whitelisted traders
    AddTrader {
        trader: String,
    },
    /// Removes the provided address from the whitelisted traders
    RemoveTrader {
        trader: String,
    },
    /// Updates the VAULT_ASSETS map
    UpdateAssets {
        to_add: Vec<VaultAsset>,
        to_remove: Vec<AssetInfo>,
    },
    // Idea: Add function that handles accounting. 
    // Before anything this function gets called and it sets the VaultAsset.asset.amount to what the expected amount after the 
    // planned action is. A callback then checks if these are correct after the action took place.   
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the treasury Config
    Config {},
    /// Returns the total value of all held assets 
    TotalValue {},
    // Returns the value of one specific asset
    HoldingValue { identifier: String },
    // Returns the amount of specified tokens this contract holds
    HoldingAmount { identifier: String },
    /// Returns the VAULT_ASSETS value for the specified key
    VaultAssetConfig { identifier: String },
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

/// Constructs the treasury traderaction message used by all dApps.
pub fn send_to_treasury(
    msgs: Vec<CosmosMsg>,
    treasury_address: &Addr,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: treasury_address.to_string(),
        msg: to_binary(&ExecuteMsg::TraderAction { msgs })?,
        funds: vec![],
    }))
}
