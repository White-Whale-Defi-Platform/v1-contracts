use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use white_whale::treasury::dapp_base::msg::{BaseExecuteMsg, BaseQueryMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    ProvideLiquidity {
        pool_id: String,
        main_asset_id: String,
        amount: Uint128,
    },
    DetailedProvideLiquidity {
        assets: Vec<(String, Uint128)>,
        pool_id: String,
        slippage_tolerance: Option<Decimal>,
    },
    WithdrawLiquidity {
        lp_token_id: String,
        amount: Uint128,
    },
    SwapAsset {
        offer_id: String,
        pool_id: String,
        amount: Uint128,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(BaseQueryMsg),
}
