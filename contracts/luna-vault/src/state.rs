use cosmwasm_std::{Addr, Order, Storage, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::{Bound, Item, Map, U64Key};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use white_whale::deposit_info::DepositInfo;
use white_whale::fee::VaultFee;
use white_whale::luna_vault::msg::UnbondHistoryResponse;

use crate::contract::VaultResult;
use crate::deserializer::deserialize_key;
use crate::error::LunaVaultError;
use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The luna-vault State contains configuration options for the vault including
// the address of the pool to trade in as well as some other addresses
pub struct State {
    pub bluna_address: Addr,
    /// The address of the liquidity pool to provide bLuna-Luna assets to for passive income
    pub astro_lp_address: Addr,
    /// The address of the Astroport factory
    pub astro_factory_address: Addr,
    pub memory_address: Addr,
    pub whitelisted_contracts: Vec<Addr>,
    pub allow_non_whitelisted: bool,
    // as a duration in seconds
    pub unbonding_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProfitCheck {
    pub last_balance: Uint128,
    pub last_profit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentBatch {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondHistory {
    pub batch_id: u64,
    pub time: u64,
    pub amount: Uint128,
    pub released: bool,
}

impl UnbondHistory {
    pub fn as_res(&self) -> UnbondHistoryResponse {
        UnbondHistoryResponse {
            batch_id: self.batch_id,
            time: self.time,
            amount: self.amount,
            released: self.released,
        }
    }
}

pub type UnbondRequest = Vec<(u64, Uint128)>;

pub const PROFIT: Item<ProfitCheck> = Item::new("\u{0}{6}profit");
pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL_INFO: Item<PoolInfoRaw> = Item::new("\u{0}{4}pool");
pub const DEPOSIT_INFO: Item<DepositInfo> = Item::new("\u{0}{7}deposit");
pub const FEE: Item<VaultFee> = Item::new("\u{0}{3}fee");
pub const UNBOND_WAITLIST: Map<(&Addr, U64Key), Uint128> = Map::new("unbond_waitlist");
pub const UNBOND_HISTORY: Map<U64Key, UnbondHistory> = Map::new("unbond_history");
pub const CURRENT_BATCH: Item<CurrentBatch> = Item::new("current_batch");

/// Store unbond wait list for the user
/// HashMap<user's address + batch_id, refund_amount>
pub fn store_unbond_wait_list(
    storage: &mut dyn Storage,
    batch_id: u64,
    sender_addr: &Addr,
    amount: Uint128,
) -> VaultResult<()> {
    UNBOND_WAITLIST.update(
        storage,
        (sender_addr, batch_id.into()),
        |existing_amount: Option<Uint128>| -> VaultResult<_> {
            Ok(existing_amount.unwrap_or_default() + amount)
        },
    )?;
    Ok(())
}

/// Stores an [UnbondHistory] with a given [batch_id].
pub fn store_unbond_history(
    storage: &mut dyn Storage,
    batch_id: u64,
    history: UnbondHistory,
) -> VaultResult<()> {
    Ok(UNBOND_HISTORY.save(storage, batch_id.into(), &history)?)
}

/// Gets an unbond history by [batch_id]
pub fn get_unbond_history(storage: &dyn Storage, batch_id: u64) -> VaultResult<UnbondHistory> {
    UNBOND_HISTORY.load(storage, batch_id.into()).map_err(|_| {
        LunaVaultError::generic_err("Burn requests not found for the specified time period")
    })
}

/// Prepares next unbond batch
pub fn prepare_next_unbond_batch(storage: &mut dyn Storage) -> VaultResult<()> {
    let mut current_batch = CURRENT_BATCH.load(storage)?;
    current_batch.id += 1;
    CURRENT_BATCH.save(storage, &current_batch)?;
    Ok(())
}

const DEFAULT_UNBOND_WAITLIST_READ_LIMIT: u32 = 30u32;

/// Gets the amount of luna that is withdrawable by the user.
/// This is known by looking at the [unbound_history] time, which is registered when unbonding, and
/// comparing it with a given [withdrawable_time], which is calculated as now - unbonding period.
/// If the necessary time has passed, then allows withdrawing the funds.
/// It allows for withdrawing multiple unbonded batches at once.
pub fn get_withdrawable_amount(
    storage: &dyn Storage,
    sender_addr: &Addr,
    withdrawable_time: u64,
) -> VaultResult<Uint128> {
    let withdrawable_amount = UNBOND_WAITLIST
        .prefix(sender_addr)
        .range(storage, None, None, Order::Ascending)
        .take(DEFAULT_UNBOND_WAITLIST_READ_LIMIT as usize)
        .fold(Uint128::zero(), |acc, item| {
            let (k, v) = item.unwrap();
            let batch_id = deserialize_key::<u64>(k).unwrap();
            if let Ok(unbond_history) = get_unbond_history(storage, batch_id) {
                if withdrawable_time > unbond_history.time {
                    acc + v
                } else {
                    acc
                }
            } else {
                acc
            }
        });
    Ok(withdrawable_amount)
}

/// Gets the ids of those unbond batches that are to be withdrawn.
/// This is known by looking at the [unbound_history] time, which is registered when unbonding, and
/// comparing it with a given [withdrawable_time], which is calculated as now - unbonding period.
/// If the necessary time has passed, then returns the batch id.
pub fn get_withdrawable_unbond_batch_ids(
    storage: &dyn Storage,
    sender_addr: &Addr,
    withdrawable_time: u64,
) -> VaultResult<Vec<u64>> {
    let withdrawable_batches: Vec<u64> = UNBOND_WAITLIST
        .prefix(sender_addr)
        .range(storage, None, None, Order::Ascending)
        .take(DEFAULT_UNBOND_WAITLIST_READ_LIMIT as usize)
        .filter_map(|item| {
            let (k, _) = item.unwrap();
            let batch_id = deserialize_key::<u64>(k).unwrap();
            if let Ok(unbond_history) = get_unbond_history(storage, batch_id) {
                if withdrawable_time > unbond_history.time {
                    Some(batch_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    Ok(withdrawable_batches)
}

/// Deprecate unbond batches by marking them as released, i.e. funds have been withdrawn.
pub fn deprecate_unbond_batches(storage: &mut dyn Storage, batch_ids: Vec<u64>) -> VaultResult<()> {
    for batch_id in batch_ids {
        if let Ok(mut unbond_history) = get_unbond_history(storage, batch_id) {
            unbond_history.released = true;
            store_unbond_history(storage, batch_id, unbond_history)?;
        }
    }
    Ok(())
}

/// Get the ids of deprecated unbond batches, i.e. those that are to be released
pub fn get_deprecated_unbond_batch_ids(
    storage: &dyn Storage,
    sender_addr: &Addr,
) -> VaultResult<Vec<u64>> {
    let deprecated_batches: Vec<u64> = UNBOND_WAITLIST
        .prefix(sender_addr)
        .range(storage, None, None, Order::Ascending)
        .take(DEFAULT_UNBOND_WAITLIST_READ_LIMIT as usize)
        .filter_map(|item| {
            let (k, _) = item.unwrap();
            let batch_id = deserialize_key::<u64>(k).unwrap();
            if let Ok(unbonded_history) = get_unbond_history(storage, batch_id) {
                if unbonded_history.released {
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

/// Remove unbond batch id from user's wait list.
pub fn remove_unbond_wait_list(
    storage: &mut dyn Storage,
    batch_ids: Vec<u64>,
    sender_addr: &Addr,
) -> VaultResult<()> {
    for batch_id in batch_ids {
        UNBOND_WAITLIST.remove(storage, (sender_addr, batch_id.into()));
    }
    Ok(())
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 10;

/// Returns all unbond_history from UnbondHistory map
#[allow(clippy::needless_lifetimes)]
pub fn all_unbond_history(
    storage: &dyn Storage,
    start: Option<u64>,
    limit: Option<u32>,
) -> VaultResult<Vec<UnbondHistory>> {
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

/// Returns unbond requests for a given address.
pub fn get_unbond_requests(
    storage: &dyn Storage,
    sender_addr: &Addr,
    start: Option<u64>,
    limit: Option<u32>,
) -> VaultResult<UnbondRequest> {
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
