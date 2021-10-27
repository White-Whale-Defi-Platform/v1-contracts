use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::Addr;
use whitewhale_liquidation_helpers::nft_minter::{LiquidationHelpersInfo};


pub const CONFIG: Item<Config> = Item::new("config");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub whitewhale_liquidators: Vec<LiquidationHelpersInfo>,
}












