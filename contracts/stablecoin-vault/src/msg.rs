use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Coin, Decimal, Uint128};
use terraswap::asset::{Asset, AssetInfo};
use cw20::Cw20ReceiveMsg;
use white_whale::fee::{Fee, CappedFee};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_address: String,
    pub anchor_money_market_address: String,
    pub aust_address: String,
    pub seignorage_address: String,
    pub profit_check_address: String,
    pub community_fund_addr: String,
    pub warchest_addr: String,
    pub asset_info: AssetInfo,
    pub token_code_id: u64,
    pub warchest_fee: Decimal,
    pub community_fund_fee: Decimal,
    pub max_community_fund_fee: Uint128,
    pub stable_cap: Uint128,
    pub vault_lp_token_name: Option<String>,
    pub vault_lp_token_symbol: Option<String>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    AbovePeg { amount: Coin, slippage: Decimal, belief_price: Decimal },
    BelowPeg { amount: Coin, slippage: Decimal, belief_price: Decimal },
    ProvideLiquidity {
        asset: Asset
    },
    SetStableCap { stable_cap: Uint128 },
    SetFee{
        community_fund_fee: Option<CappedFee>,
        warchest_fee: Option<Fee>,
     },
    SetAdmin{ admin: String },
    SetTrader{ trader: String },
    
    Callback(CallbackMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    AfterSuccessfulTradeCallback {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 3],
    pub total_value_in_ust: Uint128,
    pub total_share: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    AssertLimitOrder{ offer_coin: Coin, ask_denom: String, minimum_receive: Uint128 },
}