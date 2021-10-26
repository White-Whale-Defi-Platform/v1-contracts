use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg,  StdResult, Uint128, WasmMsg, 
};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    FlashLoan {
        payload: FlashLoanPayload,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FlashLoanPayload {
    pub requested_asset: Asset,
    pub callback: Binary,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    Token { contract_addr: String },
    NativeToken { denom: String },
}

/// @dev Returns CosmosMsg struct which executes FlashLoan function on the UST Vault arb 
/// @param ust_vault_contract : Vault contract address
/// @param stable_denom : UST denom
/// @param amount : UST amount to borrow for the tx
/// @param callback_binary : CallbackMsg coded into binary 
pub fn build_flash_loan_msg(
    ust_vault_contract: String,
    stable_denom: String,
    amount: Uint256,
    callback_binary: Binary,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: ust_vault_contract,
        msg: to_binary(&ExecuteMsg::FlashLoan {
            payload : FlashLoanPayload {
                requested_asset: Asset {
                    info : AssetInfo::NativeToken { denom : stable_denom } ,
                    amount: amount.into()
                } ,
                callback: callback_binary,
            } 
        })?,
        funds: vec![],
    }))
}