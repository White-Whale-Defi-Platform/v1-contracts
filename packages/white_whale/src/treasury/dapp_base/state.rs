use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Deps, StdResult};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use terraswap::asset::AssetInfo;

use crate::{denom::is_denom, memory::item::Memory};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// The BaseState contains the main addresses needed for sending and verifying messages
pub struct BaseState {
    pub treasury_address: Addr,
    pub trader: Addr,
    pub memory: Memory,
}

pub const BASESTATE: Item<BaseState> = Item::new("\u{0}{10}base_state");
// Every DApp should use the provide memory contract for token/contract address resolution
pub const ADMIN: Admin = Admin::new("admin");

// TODO: remove
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


// Returns the asset info for an address book entry.
pub fn get_asset_info(deps: Deps, id: &str) -> StdResult<AssetInfo> {
    let address_or_denom = ADDRESS_BOOK.load(deps.storage, id)?;
    return if is_denom(address_or_denom.as_str()) {
        Ok(AssetInfo::NativeToken {
            denom: address_or_denom,
        })
    } else {
        deps.api.addr_validate(address_or_denom.as_str())?;
        Ok(AssetInfo::Token {
            contract_addr: address_or_denom,
        })
    };
}
