use cosmwasm_std::{Addr, Decimal, Order, StdError, StdResult, Storage, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::{Bound, Item, Map, U64Key};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use white_whale::deposit_info::DepositInfo;
use white_whale::fee::VaultFee;

use crate::deserializer::deserialize_key;
use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The luna-vault State contains configuration options for the vault including
// the address of the pool to trade in as well as some other addresses
pub struct State {
    pub bluna_address: Addr,
    pub astro_lp_address: Addr,
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

impl State {
    pub fn update_exchange_rate(&mut self, total_issued: Uint128, requested_with_fee: Uint128) {
        let actual_supply = total_issued + requested_with_fee;
        if self.total_bond_amount.is_zero() || actual_supply.is_zero() {
            self.exchange_rate = Decimal::one()
        } else {
            self.exchange_rate = Decimal::from_ratio(self.total_bond_amount, actual_supply);
        }
    }
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
    pub epoch_period: u64,
    // as a duration in seconds
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,
    // as a duration in seconds
    pub peg_recovery_fee: Decimal,
    // must be in [0, 1].
    pub er_threshold: Decimal, // exchange rate threshold. Must be in [0, 1].
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

/// Store undelegation wait list per each batch
/// HashMap<user's address + batch_id, requested_amount>
pub fn store_unbond_wait_list(
    storage: &mut dyn Storage,
    batch_id: u64,
    sender_addr: &Addr,
    amount: Uint128,
) -> StdResult<()> {
    UNBOND_WAITLIST.update(
        storage,
        (sender_addr, batch_id.into()),
        |existing_amount: Option<Uint128>| -> StdResult<_> {
            Ok(existing_amount.unwrap_or_default() + amount)
        },
    )?;
    Ok(())
}

pub fn store_unbond_history(
    storage: &mut dyn Storage,
    batch_id: u64,
    history: UnbondHistory,
) -> StdResult<()> {
    UNBOND_HISTORY.save(storage, batch_id.into(), &history)
}

pub fn read_unbond_history(storage: &dyn Storage, epoc_id: u64) -> StdResult<UnbondHistory> {
    UNBOND_HISTORY
        .load(storage, epoc_id.into())
        .map_err(|_| StdError::generic_err("Burn requests not found for the specified time period"))
}

const DEFAULT_UNBOND_WAITLIST_READ_LIMIT: u32 = 30u32;

/// Return all requested unbond amount.
/// This needs to be called after process withdraw rate function.
/// If the batch is released, this will return user's requested
/// amount proportional to withdraw rate.
pub fn get_finished_amount(
    storage: &dyn Storage,
    sender_addr: &Addr,
    limit: Option<u32>,
) -> StdResult<Uint128> {
    let withdrawable_amount = UNBOND_WAITLIST
        .prefix(sender_addr)
        .range(storage, None, None, Order::Ascending)
        .take(
            limit
                .unwrap_or(DEFAULT_UNBOND_WAITLIST_READ_LIMIT)
                .min(MAX_LIMIT) as usize,
        )
        .fold(Uint128::zero(), |acc, item| {
            let (k, v) = item.unwrap();
            let batch_id = deserialize_key::<u64>(k).unwrap();
            if let Ok(h) = read_unbond_history(storage, batch_id) {
                if h.released {
                    acc + v * h.withdraw_rate
                } else {
                    acc
                }
            } else {
                acc
            }
        });
    Ok(withdrawable_amount)
}

pub fn get_unbond_batches(
    storage: &dyn Storage,
    sender_addr: &Addr,
    limit: Option<u32>,
) -> StdResult<Vec<u64>> {
    let deprecated_batches: Vec<u64> = UNBOND_WAITLIST
        .prefix(sender_addr)
        .range(storage, None, None, Order::Ascending)
        .take(
            limit
                .unwrap_or(DEFAULT_UNBOND_WAITLIST_READ_LIMIT)
                .min(MAX_LIMIT) as usize,
        )
        .filter_map(|item| {
            let (k, _) = item.unwrap();
            let batch_id = deserialize_key::<u64>(k).unwrap();
            if let Ok(h) = read_unbond_history(storage, batch_id) {
                if h.released {
                    Some(batch_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    Ok(deprecated_batches)
}

/// Remove unbond batch id from user's wait list
pub fn remove_unbond_wait_list(
    storage: &mut dyn Storage,
    batch_id: Vec<u64>,
    sender_addr: &Addr,
) -> StdResult<()> {
    for b in batch_id {
        UNBOND_WAITLIST.remove(storage, (sender_addr, b.into()));
    }
    Ok(())
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 10;

/// Return all unbond_history from UnbondHistory map
#[allow(clippy::needless_lifetimes)]
pub fn all_unbond_history(
    storage: &dyn Storage,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<UnbondHistory>> {
    let start = U64Key::from(start.unwrap_or_default());
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let res = UNBOND_HISTORY
        .range(
            storage,
            Some(Bound::Exclusive(start.into())),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|item| {
            let history: UnbondHistory = item.unwrap().1;
            Ok(history)
        })
        .collect();
    res
}

/// Return the finished amount for all batches that has been before the given block time.
pub fn query_get_finished_amount(
    storage: &dyn Storage,
    sender_addr: &Addr,
    block_time: u64,
    limit: Option<u32>,
) -> StdResult<Uint128> {
    let withdrawable_amount = UNBOND_WAITLIST
        .prefix(sender_addr)
        .range(storage, None, None, Order::Ascending)
        .take(
            limit
                .unwrap_or(DEFAULT_UNBOND_WAITLIST_READ_LIMIT)
                .min(MAX_LIMIT) as usize,
        )
        .fold(Uint128::zero(), |acc, item| {
            let (k, v) = item.unwrap();
            let batch_id = deserialize_key::<u64>(k).unwrap();
            if let Ok(h) = read_unbond_history(storage, batch_id) {
                if h.time < block_time {
                    acc + v * h.withdraw_rate
                } else {
                    acc
                }
            } else {
                acc
            }
        });
    Ok(withdrawable_amount)
}

pub fn get_unbond_requests(
    storage: &dyn Storage,
    sender_addr: &Addr,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<UnbondRequest> {
    let start = U64Key::from(start.unwrap_or_default());

    let sender_requests: Vec<_> = UNBOND_WAITLIST
        .prefix(sender_addr)
        .range(
            storage,
            Some(Bound::Exclusive(start.into())),
            None,
            Order::Ascending,
        )
        .take(
            limit
                .unwrap_or(DEFAULT_UNBOND_WAITLIST_READ_LIMIT)
                .min(MAX_LIMIT) as usize,
        )
        .map(|item| {
            let (k, v) = item.unwrap();
            let batch_id = deserialize_key::<u64>(k).unwrap();
            (batch_id, v)
        })
        .collect();
    Ok(sender_requests)
}
