use cosmwasm_bignumber::Uint256;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use whitewhale_liquidation_helpers::dex_arb_helper::PoolInfo;

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");

pub const LOOP: Item<Loop> = Item::new("loop");
pub const TERRASWAP: Item<Terraswap> = Item::new("terraswap");
pub const ASTROPORT: Item<Astroport> = Item::new("astroport");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub ust_vault_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Loop {
    pub loop_pools: Vec<PoolInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Terraswap {
    pub terraswap_pools: Vec<PoolInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Astroport {
    pub astroport_pools: Vec<PoolInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_arbs: u64,
    pub total_ust_profit: Uint256,
}
