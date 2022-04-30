use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stores the state of the contract, including the current owner of the unbond handler, the expiration time for when
/// liquidations are allowed, and the address of the memory contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Option<Addr>,
    pub expiration_time: Option<u64>,
    pub memory_contract: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const ADMIN: Admin = Admin::new("admin");
