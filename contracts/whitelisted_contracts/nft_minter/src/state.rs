use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item};
use cosmwasm_std::Addr;
use whitewhale_liquidation_helpers::nft_minter::{LiquidationHelpersInfo};
use whitewhale_liquidation_helpers::metadata::Metadata;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TMP_NFT_INFO: Item<TmpNftInfo> = Item::new("tmp_nft_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub cw721_code_id: u64,
    pub whitewhale_liquidators: Vec<LiquidationHelpersInfo>,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TmpNftInfo {
    pub liquidator_addr: String,
    pub metadata: Metadata,
    pub token_uri: String
}










