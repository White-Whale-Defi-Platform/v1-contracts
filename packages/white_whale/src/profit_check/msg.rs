use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub vault_address: HumanAddr,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    BeforeTrade {},
    AfterTrade {},
    SetVault{ vault_address: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LastBalance{},
    Vault{},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastBalanceResponse {
    pub last_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultResponse {
    pub vault_address: HumanAddr,
}