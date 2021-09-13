use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Coin, Decimal, Uint128};
use terraswap::asset::{Asset, AssetInfo};
use cw20::Cw20ReceiveMsg;


// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct AssertMinimumReceive {
//     pub asset_info: AssetInfo,
//     pub prev_balance: Uint128,
//     pub minimum_receive: Uint128,
//     pub receiver: Addr,
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_address: String,
    pub anchor_money_market_address: String,
    pub aust_address: String,
    pub seignorage_address: String,
    pub profit_check_address: String,
    pub burn_addr: String,
    pub profit_burn_ratio: Decimal,
    pub asset_info: AssetInfo,
    pub slippage: Decimal,
    pub token_code_id: u64
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Cw20ReceiveMsg),
    AbovePeg { amount: Coin, uaust_withdraw_amount: Uint128 },
    BelowPeg { amount: Coin, uaust_withdraw_amount: Uint128 },
    ProvideLiquidity {
        asset: Asset
    },
    AnchorDeposit { amount: Coin },
    SetSlippage { slippage: Decimal },
    SetBurnAddress{ burn_addr: String }
}

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