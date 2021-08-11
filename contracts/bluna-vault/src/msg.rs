use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Coin, Decimal, Uint128};
use terraswap::asset::Asset;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_address: String,
    pub bluna_hub_address: String,
    pub bluna_address: String,
    pub slippage: Decimal,
    pub token_code_id: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Cw20ReceiveMsg),
    // Burn{ amount: Coin },
    // Claim{ amount: Coin },
    // Mint{ amount: Coin },
    Swap{ amount: Coin },
    ProvideLiquidity {
        asset: Asset
    },
    SetSlippage { slippage: Decimal }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 2],
    pub total_share: Uint128,
}