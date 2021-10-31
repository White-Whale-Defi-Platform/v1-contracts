use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_bignumber::{Uint256};
use crate::metadata::Metadata;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub cw721_code_id: u64
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateOwner {
    pub owner: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner { 
        owner: String,
    },
    AddLiquidator { 
        new_liquidator: String,
        metadata: Metadata,
        symbol: String,
        token_uri: String
    },
    UpdateLiquidator { 
        cur_liquidator: String,
        new_liquidator: Option<String>,
        metadata: Option<Metadata>,
        token_uri: Option<String>
    },
    MintNft {
        user_address: String,
        liquidated_amount: Uint256
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub whitewhale_liquidators: Vec<LiquidationHelpersInfo>,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationHelpersInfo {
    pub liquidator_contract: String,
    pub nft_contract_addr: String,
    pub total_minted: u64,
    pub metadata: Metadata,
    pub token_uri: String,
}
