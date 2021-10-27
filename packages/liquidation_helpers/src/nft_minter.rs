use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_bignumber::{Uint256};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
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
        new_liquidator: LiquidationHelpersInfo,
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
    pub total_minted: u64
}
