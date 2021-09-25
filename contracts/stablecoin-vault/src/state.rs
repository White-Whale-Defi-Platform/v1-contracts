use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Uint128};
use cw_storage_plus::{Item};
use cw_controllers::Admin;

use white_whale::deposit_info::DepositInfo;
use white_whale::fee::VaultFee;
use white_whale::deposit_manager::DepositManager;

use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub trader: CanonicalAddr,
    pub pool_address: CanonicalAddr,
    pub anchor_money_market_address: CanonicalAddr,
    pub aust_address: CanonicalAddr,
    pub seignorage_address: CanonicalAddr,
    pub profit_check_address: CanonicalAddr,
    pub anchor_min_withdraw_amount: Uint128
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL_INFO: Item<PoolInfoRaw> = Item::new("\u{0}{4}pool");
pub const DEPOSIT_INFO: Item<DepositInfo> = Item::new("\u{0}{7}deposit");
pub const FEE: Item<VaultFee> = Item::new("\u{0}{12}fee");
pub const DEPOSIT_MANAGER: DepositManager = DepositManager::new();