use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Coin, Decimal, HumanAddr, Uint128};
use terraswap::asset::Asset;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_address: HumanAddr,
    pub bluna_hub_address: HumanAddr,
    pub bluna_address: HumanAddr,
    pub slippage: Decimal,
    /// Token contract code id for initialization
    pub token_code_id: u64
    // Hook for post initalization
    // pub init_hook: Option<InitHook>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Cw20ReceiveMsg),
    // Burn{ amount: Coin },
    // Claim{ amount: Coin },
    // Mint{ amount: Coin },
    Swap{ amount: Coin },
    PostInitialize {},
    ProvideLiquidity {
        asset: Asset
    },
    SetSlippage { slippage: Decimal }
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 2],
    pub total_share: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    AssertLimitOrder{ offer_coin: Coin, ask_denom: String, minimum_receive: Uint128 },
}

pub fn create_assert_limit_order_msg(
    offer_coin: Coin,
    ask_denom: String,
    minimum_receive: Uint128
) -> Message {
    Message::AssertLimitOrder{
        offer_coin,
        ask_denom,
        minimum_receive: minimum_receive * Decimal::from_ratio(Uint128(99), Uint128(100))
    }
}
