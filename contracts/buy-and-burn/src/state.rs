use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr};
use cw_storage_plus::{Item};

pub static CONFIG_KEY: &[u8] = b"config";
pub static UST_DENOM: &str = "uust";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner_addr: CanonicalAddr,
    pub whale_token_addr: CanonicalAddr,
    pub whale_pool_addr: CanonicalAddr,
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
