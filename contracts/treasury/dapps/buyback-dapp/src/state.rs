use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// State stores LP token address
/// BaseState is initialized in contract
pub struct State {
    pub whale_vust_lp: Addr,
    pub vust_token: Addr,
    pub whale_token: Addr,
}


pub const STATE: Item<State> = Item::new("\u{0}{5}state");
