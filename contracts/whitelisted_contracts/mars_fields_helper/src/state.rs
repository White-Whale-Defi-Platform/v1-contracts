use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::Addr;


pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub ust_vault_address: Addr,
    pub fields_addresses: Vec<String>,
    pub stable_denom: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_liquidations: u64,
    pub total_ust_profit: Uint256
}










