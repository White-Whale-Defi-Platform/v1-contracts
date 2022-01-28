use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use white_whale::deposit_info::DepositInfo;
use white_whale::fee::VaultFee;

use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The Stablecoin-vault State contains configuration options for the vault including
// the address of the pool to trade in as well as some other addresses
pub struct State {
    pub anchor_money_market_address: Addr,
    pub aust_address: Addr,
    pub profit_check_address: Addr,
    pub whitelisted_contracts: Vec<Addr>,
    pub allow_non_whitelisted: bool,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL_INFO: Item<PoolInfoRaw> = Item::new("\u{0}{4}pool");
pub const DEPOSIT_INFO: Item<DepositInfo> = Item::new("\u{0}{7}deposit");
pub const FEE: Item<VaultFee> = Item::new("\u{0}{3}fee");
