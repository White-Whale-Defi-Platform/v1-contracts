use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

//----------------------------------------------------------------------------------------
// Struct's :: Contract State
//----------------------------------------------------------------------------------------

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// WHALE Token address
    pub whale_token: Addr,
    ///  S-WHALE Token address
    pub staked_whale_token: Addr,
    /// Staking contract address
    pub whale_staking_contract: Addr,
}
