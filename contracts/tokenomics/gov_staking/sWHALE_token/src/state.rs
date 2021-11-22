use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");
pub const REBASES: Map<u64, Rebase> = Map::new("rebases");
pub const GON_BALANCES: Map<&Addr, Uint128> = Map::new("gon_balances");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub initializer: Option<Addr>,
    pub staking_contract_address: Option<Addr>,
    pub rebase_tracker: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Rebase {
    pub epoch: u64,
    pub rebase: u64,
    pub totalStakedBefore: Uint128,
    pub totalStakedAfter: Uint128,
    pub amountRebased: u64,
    pub index: Decimal,
    pub blockNumberOccured: u64,
}
