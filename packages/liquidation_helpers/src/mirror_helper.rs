use cosmwasm_std::{
    to_binary, Addr, CosmosMsg,StdResult, WasmMsg,Uint128
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_bignumber::{Decimal256, Uint256};

use crate::asset::{AssetInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub ust_vault_address: String,
    pub mirror_mint_contract: String,
    pub stable_denom: String,
    pub massets_supported: Vec<MAssetInfo>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub ust_vault_address: Option<String>,
    pub mirror_mint_contract: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig { 
        new_config: UpdateConfigMsg
    },
    AddMasset { 
        new_masset_info: AssetInfo,
        pair_address: String,
    },
    LiquidateMirrorPosition {
        position_idx: Uint128,
        ust_to_borrow: Uint256,
        max_loss_amount: Uint256
    },
    /// Callbacks; only callable by the contract itself.
    Callback(CallbackMsg),
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    InitiateLiquidationCallback {
        position_idx: Uint128,
        minted_masset: Addr, 
        minted_pair_addr: String,           
        collateral_masset: AssetInfo,
        collateral_pair_addr: String,
        max_loss_amount: Uint256,
    },
    AftermAssetBuyCallback {
        position_idx: Uint128,
        minted_masset: Addr, 
        minted_pair_addr: String,           
        collateral_masset: AssetInfo,
        collateral_pair_addr: String,
        ust_amount: Uint256,
        max_loss_amount: Uint256,
    },
    AfterLiquidationCallback {
        minted_masset: Addr,   
        minted_pair_addr: String,                  
        collateral_masset: AssetInfo,
        collateral_pair_addr: String,
        ust_amount: Uint256,
        max_loss_amount: Uint256,
    },
    AftermAssetsSellCallback {
        ust_amount: Uint256,
        max_loss_amount: Uint256
    }
}

// Modified from
// https://github.com/CosmWasm/cosmwasm-plus/blob/v0.2.3/packages/cw20/src/receiver.rs#L15
impl CallbackMsg {
    pub fn to_cosmos_msg(&self, contract_addr: &Addr) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&ExecuteMsg::Callback(self.clone()))?,
            funds: vec![],
        }))
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub ust_vault_address: String,
    pub mirror_mint_contract: String,
    pub stable_denom: String,
    pub massets_supported: Vec<MAssetInfo>
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_liquidations: u64,
    pub total_ust_profit: Uint256,
}







#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MAssetInfo {
    pub asset_token: AssetInfo,
    pub pair_address: Addr,
    pub auction_discount: Decimal256,
    pub min_collateral_ratio: Decimal256,    
}