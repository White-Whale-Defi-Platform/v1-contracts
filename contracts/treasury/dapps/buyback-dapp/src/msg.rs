use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use white_whale::treasury::dapp_base::msg::BaseInstantiateMsg;

use white_whale::treasury::dapp_base::msg::{BaseExecuteMsg, BaseQueryMsg};
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, Env, Fraction, MessageInfo, Response, Uint128,
    WasmMsg,
};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    // Add dapp-specific messages here
    Buyback{amount: Uint128}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(BaseQueryMsg),
    // Add dapp-specific queries here
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base: BaseInstantiateMsg,
    pub whale_vust_lp: Addr,
    pub vust_token: Addr,
    pub whale_token: Addr,
}