use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cosmwasm_std::{Binary, CanonicalAddr, Decimal, Storage, Uint128};
use cosmwasm_storage::{
    bucket_read, bucket, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

use std::cmp::Ordering;

static KEY_CONFIG: &[u8] = b"config";
static KEY_STATE: &[u8] = b"state";

static PREFIX_POLL_INDEXER: &[u8] = b"poll_indexer";
static PREFIX_POLL: &[u8] = b"poll";


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub whale_token: CanonicalAddr,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub contract_addr: CanonicalAddr,
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

pub const STATE: Item<State> = Item::new("state");

// State related to Poll Execution
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PollExecuteMsg {
    pub order: u64,
    pub contract: String,
    pub msg: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct ExecuteData {
    pub order: u64,
    pub contract: CanonicalAddr,
    pub msg: Binary,
}

impl Eq for ExecuteData {}

impl Ord for ExecuteData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for ExecuteData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExecuteData {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

// State related to Polls
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub status: PollStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub execute_data: Option<Vec<ExecuteData>>,
    pub deposit_amount: Uint128,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,
    pub staked_amount: Option<Uint128>,
}

// State objects here are good candidates to move to the packages module 
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    InProgress,
    Passed,
    Rejected,
    Executed,
    Expired,
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn config_store(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}

pub fn state_store(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, KEY_STATE)
}

pub fn state_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, KEY_STATE)
}

pub fn poll_store(storage: &mut dyn Storage) -> Bucket<Poll> {
    bucket(storage, PREFIX_POLL)
}

pub fn poll_read(storage: &dyn Storage) -> ReadonlyBucket<Poll> {
    bucket_read(storage, PREFIX_POLL)
}

pub fn poll_indexer_store<'a>(
    storage: &'a mut dyn Storage,
    status: &PollStatus,
) -> Bucket<'a, bool> {
    Bucket::multilevel(
        storage,
        &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
    )
}