use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use terraswap::asset::AssetInfo;
use white_whale::{treasury::dapp_base::state::BaseState, fee::Fee};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// State stores Base State and LP token address
pub struct State {
    pub base: BaseState,
    pub lp_token_addr: Addr,
    pub memory_addr: Addr,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// State stores Base State and LP token address
pub struct Pool {
    pub deposit_asset: AssetInfo,
    pub assets: Vec<String>,
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL: Item<Pool> = Item::new("\u{0}{4}pool");
pub const FEE: Item<Fee> = Item::new("\u{0}{3}fee");
