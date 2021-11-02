use cw_controllers::Admin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr};
use cw_storage_plus::{Item, Map};

use crate::vault_assets::VaultAsset;
use terraswap::asset::{Asset, AssetInfoRaw};

pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub traders: Vec<CanonicalAddr>,
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const ADMIN: Admin = Admin::new("admin");
pub const VAULT_ASSETS: Map<&str, VaultAsset> = Map::new("vault_assets");
