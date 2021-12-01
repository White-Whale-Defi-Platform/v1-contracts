use cosmwasm_std::{Decimal, Uint128};

use white_whale::treasury::dapp_base::msg::BaseExecuteMsg;

pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    ProvideLiquidity {
        pool_id: String,
        main_asset_id: String,
        amount: Uint128,
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
