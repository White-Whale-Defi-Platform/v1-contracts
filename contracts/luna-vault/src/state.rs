use cosmwasm_std::{Addr, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use white_whale::deposit_info::DepositInfo;
use white_whale::fee::VaultFee;

use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The luna-vault State contains configuration options for the vault including
// the address of the pool to trade in as well as some other addresses
pub struct State {
    pub bluna_address: Addr,
    pub cluna_address: Addr,
    /// The address of the liquidity pool to provide bLuna-Luna assets to for passive income
    pub astro_lp_address: Addr,
    /// The address of the Astroport factory
    pub astro_factory_address: Addr,
    pub memory_address: Addr,
    pub whitelisted_contracts: Vec<Addr>,
    pub allow_non_whitelisted: bool,
    // code id for the unbond handler contract
    pub unbond_handler_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProfitCheck {
    pub last_balance: Uint128,
    pub last_profit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondDataCache {
    pub owner: Addr,
    pub bluna_amount: Uint128,
}

pub const PROFIT: Item<ProfitCheck> = Item::new("profit");
pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("state");
pub const POOL_INFO: Item<PoolInfoRaw> = Item::new("pool");
pub const DEPOSIT_INFO: Item<DepositInfo> = Item::new("deposit");
pub const FEE: Item<VaultFee> = Item::new("fee");

// Unbond handler objects

pub type UnbondHandlerAddr = Addr;
pub type UserAddr = Addr;

// Unbond handler expiration time variable, configurable
pub const UNBOND_HANDLER_EXPIRATION_TIME: Item<u64> = Item::new("unbond_handler_expiration_time");

// Unbond handler addresses that are available and ready to be used
pub const UNBOND_HANDLERS_AVAILABLE: Item<Vec<Addr>> = Item::new("unbond_handlers_available");
// Map of unbond handlers assigned to user addresses
pub const UNBOND_HANDLERS_ASSIGNED: Map<UserAddr, UnbondHandlerAddr> =
    Map::new("unbond_handlers_assigned");
// Map of expiration times for a unbond handlers addresses
pub const UNBOND_HANDLER_EXPIRATION_TIMES: Map<UnbondHandlerAddr, u64> =
    Map::new("unbond_handler_expiration_times");
// Cache use to temporarily store [UnbondDataCache] when no handler are available and a new one
// needs to be created. This cache will be used by the reply handler.
pub const UNBOND_CACHE: Item<UnbondDataCache> = Item::new("unbond_cache");

// 40 days
pub const DEFAULT_UNBOND_EXPIRATION_TIME: u64 = 3456000u64;
pub const UNBOND_HANDLER_EXPIRATION_TIMES_READ_LIMIT: u32 = 30u32;
