use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::Addr;
use whitewhale_periphery::mirror_helper::{ MAssetInfo };

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub controller_strategy: Addr,
    pub mirror_mint_contract: Addr,
    pub stable_denom: String,
    pub massets_supported: Vec<MAssetInfo>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_liquidations: u64,
    pub total_ust_profit: Uint256
}










