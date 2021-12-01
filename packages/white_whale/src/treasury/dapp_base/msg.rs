use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BaseInstantiateMsg {
    pub treasury_address: String,
    pub trader: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaseExecuteMsg {
    UpdateConfig {
        treasury_address: Option<String>,
        trader: Option<String>,
    },
    UpdateAddressBook {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    SetAdmin {
        admin: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaseQueryMsg {
    Config {},
    AddressBook { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BaseStateResponse {
    pub treasury_address: String,
    pub trader: String,
}
