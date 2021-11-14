use cw_controllers::Admin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr, Decimal, Deps, Env, Fraction, StdResult, Uint128};
use cw_storage_plus::{Item, Map};

use crate::query::terraswap::query_pool;
use crate::treasury::vault_assets::{get_identifier, VaultAsset};
use terraswap::asset::AssetInfoRaw;
use terraswap::pair::PoolResponse;

pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub traders: Vec<CanonicalAddr>,
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const ADMIN: Admin = Admin::new("admin");
pub const VAULT_ASSETS: Map<&str, VaultAsset> = Map::new("vault_assets");

pub fn lp_value(deps: Deps, env: &Env, pool_addr: &Addr, holdings: Uint128) -> StdResult<Uint128> {
    // Get LP pool info
    let pool_info: PoolResponse = query_pool(deps, pool_addr)?;

    // Get total supply of LP tokens and calculate share
    let total_lp = pool_info.total_share;
    let share = holdings / total_lp;

    let asset_1 = &pool_info.assets[0];
    let asset_2 = &pool_info.assets[1];

    // load the assets
    let mut vault_asset_1: VaultAsset =
        VAULT_ASSETS.load(deps.storage, get_identifier(&asset_1.info).as_str())?;
    let mut vault_asset_2: VaultAsset =
        VAULT_ASSETS.load(deps.storage, get_identifier(&asset_2.info).as_str())?;

    // set the amounts to the LP holdings
    let vault_asset_1_amount = share * asset_1.amount;
    let vault_asset_2_amount = share * asset_2.amount;

    // Call value on these assets.
    Ok(vault_asset_1.value(deps, env, Some(vault_asset_1_amount))?
        + vault_asset_2.value(deps, env, Some(vault_asset_2_amount))?)
}

pub fn proxy_value(
    deps: Deps,
    env: &Env,
    proxy_asset: &AssetInfoRaw,
    mut multiplier: Decimal,
    proxy_pool: &Option<Addr>,
    holding: Uint128,
) -> StdResult<Uint128> {
    match proxy_pool {
        // If there is a proxy pool, use this pool to set the multiplier using the ratio and
        // vault asset info
        Some(pool_addr) => {
            // example: pool_addr is bluna/luna pool.
            // holding = #bluna
            let pool_info: PoolResponse = query_pool(deps, pool_addr)?;

            let asset_1 = &pool_info.assets[0];
            let asset_2 = &pool_info.assets[1];
            let ratio = Decimal::from_ratio(asset_1.amount, asset_2.amount);

            if asset_1.info == proxy_asset.to_normal(deps.api)? {
                // asset_1 is luna
                // luna/bluna
                multiplier = ratio;
            } else {
                // asset_1 is bluna
                // luna/bluna
                multiplier = ratio.inv().unwrap_or_default();
            }

            let proxy_holding = holding * multiplier;
            let mut proxy_asset: VaultAsset = VAULT_ASSETS.load(
                deps.storage,
                get_identifier(&proxy_asset.to_normal(deps.api)?).as_str(),
            )?;
            proxy_asset.value(deps, env, Some(proxy_holding))
        }
        // If no proxy pool is given, use the current multiplier
        None => {
            let mut proxy_vault_asset: VaultAsset = VAULT_ASSETS.load(
                deps.storage,
                get_identifier(&proxy_asset.to_normal(deps.api)?).as_str(),
            )?;
            proxy_vault_asset.asset.amount = holding * multiplier;
            // call value on proxy asset
            proxy_vault_asset.value(deps, env, None)
        }
    }
}
