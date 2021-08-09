use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

use crate::pool_info::PoolInfoRaw;

pub static CONFIG_KEY: &[u8] = b"config";
static KEY_PAIR_INFO: &[u8] = b"asset_info";
pub static BURN_MINT_CONTRACT: &str = "terra1z3sf42ywpuhxdh78rr5vyqxpaxa0dx657x5trs"; // seignorage contract on tequila-0004
pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub trader: CanonicalAddr,
    pub pool_address: CanonicalAddr,
    pub anchor_money_market_address: CanonicalAddr,
    pub aust_address: CanonicalAddr,
    pub seignorage_address: CanonicalAddr,
    pub profit_check_address: CanonicalAddr,
    pub slippage: Decimal,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}


pub fn store_pool_info<S: Storage>(storage: &mut S, data: &PoolInfoRaw) -> StdResult<()> {
    Singleton::new(storage, KEY_PAIR_INFO).save(data)
}

pub fn read_pool_info<S: Storage>(storage: &S) -> StdResult<PoolInfoRaw> {
    ReadonlySingleton::new(storage, KEY_PAIR_INFO).load()
}

