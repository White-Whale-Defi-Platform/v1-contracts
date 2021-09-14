use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

use white_whale::deposit_info::DepositInfo;

use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub trader: CanonicalAddr,
    pub pool_address: CanonicalAddr,
    pub anchor_money_market_address: CanonicalAddr,
    pub aust_address: CanonicalAddr,
    pub seignorage_address: CanonicalAddr,
    pub profit_check_address: CanonicalAddr,
    pub burn_addr: CanonicalAddr,
    pub profit_burn_ratio: Decimal,
    pub deposit_info: DepositInfo
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL_INFO: Item<PoolInfoRaw> = Item::new("\u{0}{4}pool");
pub const USER_DEPOSITS: Map<CanonicalAddr, Uint128> = Map::new("deposits");
