use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Decimal, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub whale_token_addr: String,
    pub whale_pair_addr: String,
    pub anchor_money_market_addr: String,
    pub aust_addr: String,
    pub anchor_deposit_threshold: Uint128,
    pub anchor_withdraw_threshold: Uint128,
    pub anchor_deposit_ratio: Decimal
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Burn{},
    Deposit {},
    BurnProfits{},
    UpdateAdmin{ admin: String },
    UpdateAnchorDepositRatio{ ratio: Decimal },
    UpdateAnchorDepositThreshold{ threshold: Uint128 },
    UpdateAnchorWithdrawThreshold{ threshold: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Admin {},
    Config {}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub token_addr: Addr,
    pub pool_addr: Addr,
    pub anchor_money_market_addr: Addr,
    pub aust_addr: Addr,
    pub anchor_deposit_threshold: Uint128,
    pub anchor_withdraw_threshold: Uint128,
    pub anchor_deposit_ratio: Decimal
}
