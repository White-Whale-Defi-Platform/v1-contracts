use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Decimal};
use cw_controllers::Admin;
use cw_storage_plus::Item;
use white_whale::{memory::item::Memory, treasury::vault_assets::VaultAsset};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub memory: Memory,
    pub rebait_ratio: Decimal,
    pub dex_arb_addr: Addr,
    pub seignorage_addr: Addr,
    pub vault_addr: Addr,
    pub whale: VaultAsset,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
