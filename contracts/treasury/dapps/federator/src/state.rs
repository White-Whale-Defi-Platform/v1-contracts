use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, Uint64};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The state contains the main addresses needed for sending and verifying messages
pub struct State {
    pub treasury_address: Addr,
    pub trader: Addr,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
/// key/value pair for contract addresses
/// <Opcode, address>
// pub const ADDRESS_BOOK: Map<&str, String> = Map::new("address_book");
/// Instruction set
pub const INSTRUCTION_SET: Map<&str, (Addr, Binary)> = Map::new("instruction_set");
