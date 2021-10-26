use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item};
use cosmwasm_bignumber::{Uint256};
use cosmwasm_std::Addr;
use whitewhale_liquidation_helpers::dex_arb_helper::{ PoolInfo};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub ust_vault_address: Addr,
    pub astroport_router: Addr,
    pub stable_denom: String,
    pub terraswap_pools: Vec<PoolInfo>,
    pub loop_pools: Vec<PoolInfo>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_arbs: u64,
    pub total_ust_profit: Uint256
}










