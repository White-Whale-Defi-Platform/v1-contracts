use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, Uint128};
use cosmwasm_storage::{ReadonlySingleton, singleton_read};
use cw_controllers::Admin;
use cw_storage_plus::Item;

static KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub whale_token_addr: CanonicalAddr,
}

pub fn state_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, KEY_STATE)
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("state");
