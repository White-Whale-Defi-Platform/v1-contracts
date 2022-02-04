use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

use white_whale::deposit_info::ArbBaseAsset;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The Arb State contains configuration options for the vault including
// the address of the pool to trade in as well as some other addresses
pub struct State {
    pub vault_address: Addr,
    pub seignorage_address: Addr,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const ARB_BASE_ASSET: Item<ArbBaseAsset> = Item::new("\u{0}{7}deposit");
pub const POOLS: Map<&str, Addr> = Map::new("pools");
