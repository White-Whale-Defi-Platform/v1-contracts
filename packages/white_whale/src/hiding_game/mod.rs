use cosmwasm_std::{Decimal, Uint128, StdResult, Addr, CosmosMsg, WasmMsg, to_binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terra_rust_script_derive::CosmWasmContract;
use terraswap::asset::AssetInfo;

use crate::treasury::vault_assets::VaultAsset;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub memory: String,
    pub rebait_ratio: Decimal,
    pub dex_arb_addr: String,
    pub seignorage_addr: String,
    pub vault_addr: String,
    pub whale: VaultAsset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the config
    UpdateConfig {
    },
    WhaleTrade {
        pair: (String, String),
        offer: String,
        amount: Uint128,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
    Callback(CallbackMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    AfterTrade { },
}

// Modified from
// https://github.com/CosmWasm/cosmwasm-plus/blob/v0.2.3/packages/cw20/src/receiver.rs#L15
impl CallbackMsg {
    pub fn to_cosmos_msg<T: Clone +std::fmt::Debug + PartialEq + JsonSchema>(
        &self,
        contract_addr: &Addr,
    ) -> StdResult<CosmosMsg<T>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&ExecuteMsg::Callback(self.clone()))?,
            funds: vec![],
        }))
    }
}