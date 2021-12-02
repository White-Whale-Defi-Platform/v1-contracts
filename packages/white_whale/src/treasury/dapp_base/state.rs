use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr, Deps, StdResult};
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
// stores name and address of tokens and pairs
// Example: pairs can be named after the LP token id.
// LP token key: "ust_luna"
// Pair key: "ust_luna_pair"
pub const ADDRESS_BOOK: Map<&str, String> = Map::new("address_book");

// Loads token address from address book. Throws error if its a native token
pub fn load_contract_addr(deps: Deps, id: &str) -> StdResult<Addr> {
    deps.api
        .addr_validate(ADDRESS_BOOK.load(deps.storage, id)?.as_str())
}
