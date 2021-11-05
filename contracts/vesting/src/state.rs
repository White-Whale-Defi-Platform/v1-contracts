use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use white_whale::vesting::{AllocationInfo, Schedule};

const PREFIX_KEY_VESTING_INFO: &[u8] = b"vesting_info";

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const ALLOCATIONS: Map<&Addr, AllocationInfo> = Map::new("vested_allocations");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Account which can create new allocations
    pub owner: Addr,
    /// Address of WHALE token
    pub whale_token: Addr,
    /// By default, unlocking starts at WhiteWhale launch, with a cliff of 12 months and a duration of 12 months.
    /// If not specified, all allocations use this default schedule
    pub default_unlock_schedule: Schedule,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// WHALE Tokens deposited into the contract
    pub total_whale_deposited: Uint128,
    /// Currently available WHALE Tokens
    pub remaining_whale_tokens: Uint128,
}
