use cosmwasm_std::{Uint128, Decimal};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terraswap::asset::Asset;
use white_whale::{treasury::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg, BaseQueryMsg}, fee::Fee};
use white_whale::treasury::msg::ValueQueryMsg;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base: BaseInstantiateMsg,
    pub token_code_id: u64,
    pub fee: Decimal,
    pub deposit_asset: String,
    pub memory_addr: String,
    pub vault_lp_token_name: Option<String>,
    pub vault_lp_token_symbol: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    // Add dapp-specific messages here
    Receive(Cw20ReceiveMsg),
    ProvideLiquidity {
        asset: Asset,
    },
    UpdatePool {
        deposit_asset: Option<String>,
        assets_to_add: Vec<String>,
        assets_to_remove: Vec<String>,
    },
    SetFee {
        fee: Fee,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(BaseQueryMsg),
    // Add dapp-specific queries here
    State {},
    ValueQuery(ValueQueryMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DepositHookMsg {
    WithdrawLiquidity {},
    ProvideLiquidity {
        asset: Asset,
    },
}