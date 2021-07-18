use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

use crate::asset::SingleInfoRaw;

pub static CONFIG_KEY: &[u8] = b"config";
static KEY_PAIR_INFO: &[u8] = b"asset_info";
pub static BURN_MINT_CONTRACT: &str = "terra1z3sf42ywpuhxdh78rr5vyqxpaxa0dx657x5trs";
pub static LUNA_UST_PAIR: &str = "terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff"; // terraswap on tequila-0004
pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub pool_address: CanonicalAddr,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}


pub fn store_lp_info<S: Storage>(storage: &mut S, data: &SingleInfoRaw) -> StdResult<()> {
    Singleton::new(storage, KEY_PAIR_INFO).save(data)
}

pub fn read_lp_info<S: Storage>(storage: &S) -> StdResult<SingleInfoRaw> {
    ReadonlySingleton::new(storage, KEY_PAIR_INFO).load()
}

