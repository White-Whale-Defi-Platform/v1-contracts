use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{AssetInfo};
use cosmwasm_std::{Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg  {
    pub whale_token_addr: String,
    pub whale_pair_addr: String, 
    pub asset_info: AssetInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg  {
    Burn {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub token_addr: Addr,
    pub pool_addr: Addr,
    pub owner: Addr,
}