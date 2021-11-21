use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

//----------------------------------------------------------------------------------------
// Struct's :: Contract State
//----------------------------------------------------------------------------------------

pub const CONFIG: Item<Config> = Item::new("config");
pub const EPOCH: Item<State> = Item::new("state");
pub const STAKER_INFO: Map<&Addr, StakerInfo> = Map::new("staker");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Account who can update config
    pub owner: Addr,
    /// WHALE Token address
    pub whale_token: Addr,
    ///  S-WHALE Token address
    pub staked_whale_token: Addr,
    pub distributor: Addr,
    pub locker: Addr,
    pub warmupContract: Addr,
    pub warmupPeriod: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Epoch {
    pub length: u64,
    pub number: u64,
    pub endBlock: u64,
    pub distribute: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub staked_amount: Uint128,
    pub gons: u64,
    pub expiry: u64,
    pub lock: bool,
}

impl Default for StakerInfo {
    fn default() -> Self {
        StakerInfo {
            staked_amount: Uint128::zero(),
            gons: Decimal::one(),
            expiry: Uint128::zero(),
            lock: false,
        }
    }
}
