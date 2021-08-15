use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cosmwasm_std::{CanonicalAddr, Decimal, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub whale_token: CanonicalAddr,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub contract_addr: CanonicalAddr,
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
