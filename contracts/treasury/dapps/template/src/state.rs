use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The state contains the main addresses needed for sending and verifying messages
pub struct State {
    pub treasury_address: CanonicalAddr,
    pub trader: CanonicalAddr,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
// stores name and address of token
pub const ADDRESS_BOOK: Map<&str, String> = Map::new("address_book");
