use std::collections::BTreeMap;

use cosmwasm_std::Addr;
use schemars::{JsonSchema, _serde_json::Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg{}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the addressbook
    UpdateContractAddresses {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    UpdateAssetAddresses {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Queries assets based on name
    QueryAssets { names: Vec<String> }, 
    QueryContracts { names: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetQueryResponse {
    pub assets: Value,
}
// BTreeMap<String, AssetInfo>
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractQueryResponse {
    pub contracts: BTreeMap<String, Addr>,
}
