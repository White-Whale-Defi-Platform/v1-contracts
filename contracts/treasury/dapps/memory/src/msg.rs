use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::AssetInfo;
use white_whale::treasury::dapp_base::msg::{BaseExecuteMsg, BaseQueryMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// dApp base messages that handle updating the config and addressbook
    Base(BaseExecuteMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Handles all the base query msgs
    Base(BaseQueryMsg),
    /// Queries assets based on name
    QueryAssets { names: Vec<String> }, // TODO: work with some composite key type
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetQueryResponse {
    pub assets: Vec<AssetInfo>,
}
