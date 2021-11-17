use cosmwasm_std::{Addr, Decimal, Deps, Env, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::query::terraswap::{query_asset_balance, query_pool};
use crate::tax::reverse_decimal;
use crate::treasury::state::*;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::PoolResponse;

// Example/contracts/mocks/mock_terraswap/terraswap_pair/src/contract.rs

/// Every VaultAsset provides a way to determine its value relative to either
/// the base asset or equivalent to a certain amount of some other asset,
/// which in its turn can be decomposed into some base asset value.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VaultAsset {
    pub asset: Asset,
    // The value reference provides the tooling to get the value of the holding
    // relative to the base asset.
    pub value_reference: Option<ValueRef>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueRef {
    // A pool address of the asset/base_asset pair
    Pool {
        pair_address: Addr,
    },
    // Liquidity pool addr to get fraction of owned liquidity
    // proxy to calculate value of both assets held by liquidity
    Liquidity {
        pool_address: Addr,
    },
    // Or a Proxy, the proxy also takes a Decimal (the multiplier)
    // Asset will be valued as if they are Proxy tokens
    Proxy {
        proxy_asset: AssetInfo,
        multiplier: Decimal,
        proxy_pool: Option<Addr>,
    },
}

impl VaultAsset {
    pub fn value(
        &mut self,
        deps: Deps,
        env: &Env,
        set_holding: Option<Uint128>,
    ) -> StdResult<Uint128> {
        // Query how many of these tokens I hold.
        //let holdings = self.asset.info.query_pool(&deps.querier, deps.api, owner_addr)?;

        let holding: Uint128;
        match set_holding {
            Some(setter) => holding = setter,
            None => {
                holding =
                    query_asset_balance(deps, &self.asset.info, env.contract.address.clone())?;
            }
        }
        self.asset.amount = holding;

        // Is there a reference to calculate the value?
        if let Some(value_reference) = self.value_reference.as_ref() {
            match value_reference {
                // A Pool refers to a swap pair that includes the base asset.
                ValueRef::Pool { pair_address } => return self.asset_value(deps, pair_address),
                ValueRef::Liquidity { pool_address } => {
                    // Check if we have a Token
                    if let AssetInfo::Token { .. } = &self.asset.info {
                        return lp_value(deps, env, pool_address, holding);
                    } else {
                        return Err(StdError::generic_err("Can't have a native LP token"));
                    }
                }
                ValueRef::Proxy {
                    proxy_asset,
                    multiplier,
                    proxy_pool,
                } => return proxy_value(deps, env, proxy_asset, *multiplier, proxy_pool, holding),
            }
        }
        // No ValueRef so this should be the base token.
        // TODO: Add error in case this is not true.
        /*if base_asset != self.asset.info {
            return Err(StdError::generic_err("No value conversion provided for this asset."));
        }*/
        Ok(holding)
    }

    /// Calculates the value of an asset compared to some base asset
    /// Requires one of the two assets to be the base asset.
    /// TODO: make it match with base asset instead of this asset
    pub fn asset_value(&self, deps: Deps, pool_addr: &Addr) -> StdResult<Uint128> {
        let pool_info: PoolResponse = query_pool(deps, pool_addr)?;
        let ratio = Decimal::from_ratio(pool_info.assets[0].amount, pool_info.assets[1].amount);
        if self.asset.info == pool_info.assets[0].info {
            Ok(self.asset.amount * reverse_decimal(ratio))
        } else {
            Ok(self.asset.amount * ratio)
        }
    }
}

/// The proxy struct acts as an Asset overwrite.
/// By setting this proxy you define the asset to be some
/// other asset while also providing the relevant pool
/// address for that asset.
/// For example: AssetInfo = bluna, BaseAsset = uusd, Proxy: Luna/ust pool
/// proxy_pool = bluna/luna, multiplier = proxy_pool bluna price
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proxy {
    // Proxy asset
    proxy_asset: AssetInfo,
    // Can be set to some constant or set to price,
    multiplier: Decimal,
    // LP pool to get multiplier
    proxy_pool: Option<Addr>,
}

impl Proxy {
    pub fn new(
        multiplier: Decimal,
        proxy_asset: AssetInfo,
        proxy_pool: Option<Addr>,
    ) -> StdResult<Self> {
        Ok(Self {
            proxy_asset,
            multiplier,
            proxy_pool,
        })
    }
}

pub fn get_identifier(asset_info: &AssetInfo) -> &String {
    match asset_info {
        AssetInfo::NativeToken { denom } => denom,
        AssetInfo::Token { contract_addr } => contract_addr,
    }
}
