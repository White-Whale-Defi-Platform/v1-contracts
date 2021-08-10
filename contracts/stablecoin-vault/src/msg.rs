use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Coin, Decimal, HumanAddr, Uint128};
use terraswap::asset::{Asset, AssetInfo};
use cw20::Cw20ReceiveMsg;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssertMinimumReceive {
    pub asset_info: AssetInfo,
    pub prev_balance: Uint128,
    pub minimum_receive: Uint128,
    pub receiver: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_address: HumanAddr,
    pub anchor_money_market_address: HumanAddr,
    pub aust_address: HumanAddr,
    pub seignorage_address: HumanAddr,
    pub profit_check_address: HumanAddr,
    pub asset_info: AssetInfo,
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
    AbovePeg { amount: Coin, uaust_withdraw_amount: Uint128 },
    BelowPeg { amount: Coin, uaust_withdraw_amount: Uint128 },
    PostInitialize {},
    ProvideLiquidity {
        asset: Asset
    },
    AnchorDeposit { amount: Coin },
    SetSlippage { slippage: Decimal }
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 3],
    pub total_deposits_in_ust: Uint128,
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
