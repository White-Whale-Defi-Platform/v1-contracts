use cosmwasm_std::{
    to_binary, Addr, CosmosMsg,StdResult, WasmMsg
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_bignumber::{Uint256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub ust_arb_strategy: String,
    pub martian_fields_addr: String,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub ust_arb_strategy: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig { 
        new_config: UpdateConfigMsg
    },
    AddFieldsStrategy {
        fields_strat_addr: String
    },
    LiquidateFieldsPosition {
        user_address: String,
        ust_to_borrow: Uint256,
        fields_strat_addr: String
    },
    /// Callbacks; only callable by the contract itself.
    Callback(CallbackMsg),
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    InitiateLiquidationCallback { 
        user_address: String,
        fields_strat_addr: String
    },
    AfterLiquidationCallback { },
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
    pub ust_arb_strategy: String,
    pub fields_addresses: Vec<String>,
    pub stable_denom: String,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_liquidations: u64,
    pub total_ust_profit: Uint256,
}






#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MartianFieldsLiquidationMsg {
    Liquidate {
        user: String,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MartianFieldsQueryMsg { 
    /// Return data on an individual user's position. Response: `PositionUnchecked`
    Position {
        user: String,
    },
    /// Query the health of a user's position: value of assets, debts, and LTV. Response: `Health`
    Health {
        user: String,
    },
}







