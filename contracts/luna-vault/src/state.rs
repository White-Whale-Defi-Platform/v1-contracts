use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map, U64Key};

use white_whale::deposit_info::DepositInfo;
use white_whale::fee::VaultFee;

use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The luna-vault State contains configuration options for the vault including
// the address of the pool to trade in as well as some other addresses
pub struct State {
    pub anchor_money_market_address: Addr,
    pub bluna_address: Addr,
    pub memory_address: Addr,
    pub whitelisted_contracts: Vec<Addr>,
    pub allow_non_whitelisted: bool,

    pub exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_vault_balance: Uint128,
    pub actual_unbonded_amount: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProfitCheck {
    pub last_balance: Uint128,
    pub last_profit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentBatch {
    pub id: u64,
    pub requested_with_fee: Uint128,
}

// The Parameters contain necessary information for unbonding vluna
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Parameters {
    pub epoch_period: u64, // as a duration in seconds
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,     // as a duration in seconds
    pub peg_recovery_fee: Decimal, // must be in [0, 1].
    pub er_threshold: Decimal,     // exchange rate threshold. Must be in [0, 1].
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondHistory {
    pub batch_id: u64,
    pub time: u64,
    pub amount: Uint128,
    pub applied_exchange_rate: Decimal,
    pub withdraw_rate: Decimal,
    pub released: bool,
}

pub type UnbondRequest = Vec<(u64, Uint128)>;

pub const PROFIT: Item<ProfitCheck> = Item::new("\u{0}{6}profit");
pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL_INFO: Item<PoolInfoRaw> = Item::new("\u{0}{4}pool");
pub const DEPOSIT_INFO: Item<DepositInfo> = Item::new("\u{0}{7}deposit");
pub const FEE: Item<VaultFee> = Item::new("\u{0}{3}fee");
pub const PARAMETERS: Item<Parameters> = Item::new("\u{0}{b}parameters");
pub const UNBOND_WAITLIST: Map<(&Addr, U64Key), Uint128> = Map::new("unbond_waitlist");
pub const UNBOND_HISTORY: Map<U64Key, UnbondHistory> = Map::new("unbond_history");
pub const CURRENT_BATCH: Item<CurrentBatch> = Item::new("current_batch");
