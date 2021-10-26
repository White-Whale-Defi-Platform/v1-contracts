use cosmwasm_std::{
    to_binary, Addr, CosmosMsg,StdResult, WasmMsg,Decimal, Uint128
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_bignumber::{Decimal256, Uint256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub ust_arb_strategy: String,
    pub red_bank_addr: String,
    pub astroport_router: String,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub ust_arb_strategy: Option<String>,
    pub red_bank_addr: Option<String>,
    pub astroport_router: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig { 
        new_config: UpdateConfigMsg
    },
    AddAsset { 
        new_asset: MarsAsset
    },
    LiquidateRedBankUser {
        user_address: String,
        ust_to_borrow: Uint256,
        debt_asset: MarsAsset,
        collateral_asset: MarsAsset,
        max_loss_amount: Uint256
    },
    /// Callbacks; only callable by the contract itself.
    Callback(CallbackMsg),
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    InitiateLiquidationCallback {
        user_address: String, 
        debt_asset: MarsAsset, 
        collateral_asset: MarsAsset, 
        max_loss_amount: Uint256    

    },
    AfterDebtAssetBuyCallback {
        user_address: String, 
        debt_asset: MarsAsset, 
        collateral_asset: MarsAsset, 
        ust_amount: Uint256,
        max_loss_amount: Uint256    
    },
    AfterLiquidationCallback {
        debt_asset: MarsAsset, 
        collateral_asset: MarsAsset, 
        ust_amount: Uint256,
        max_loss_amount: Uint256    
    },
    AfterAssetsSellCallback {
        ust_amount: Uint256,
        max_loss_amount: Uint256
    }
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
    pub red_bank_addr: String,
    pub astroport_router: String,
    pub stable_denom: String,
    pub assets_supported: Vec<RedBankAssetsInfo>
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_liquidations: u64,
    pub total_ust_profit: Uint256,
}






#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarsLiquidationMsg {
    /// Liquidate under-collateralized native loans. Coins used to repay must be sent in the
    /// transaction this call is made.
    LiquidateNative {
        /// Collateral asset liquidator gets from the borrower
        collateral_asset: MarsAsset,
        /// Denom used in Terra (e.g: uluna, uusd) of the debt asset
        debt_asset_denom: String,
        /// The address of the borrower getting liquidated
        user_address: String,
        /// Whether the liquidator gets liquidated collateral in maToken (true) or
        /// the underlying collateral asset (false)
        receive_ma_token: bool,
    },
    /// Liquidate under-collateralized cw20 loan using the sent cw20 tokens.
    LiquidateCw20 {
        /// Collateral asset liquidator gets from the borrower
        collateral_asset: MarsAsset,
        /// The address of the borrower getting liquidated
        user_address: String,
        /// Whether the liquidator gets liquidated collateral in maToken (true) or
        /// the underlying collateral asset (false)
        receive_ma_token: bool,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarsQueryMsg { 
    /// Get asset market
    Market { asset: MarsAsset },
    /// Get all debt positions for a user. Returns UsetDebtResponse
    UserDebt { user_address: String },

    /// Get user debt position for a specific asset. Returns UserAssetDebtResponse
    UserAssetDebt { user_address: String, asset: MarsAsset },

    /// Get info about whether or not user is using each asset as collateral.
    /// Returns UserCollateralResponse
    UserCollateral { user_address: String },

    /// Get user position. Returns UserPositionResponse
    UserPosition { user_address: String },    
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RedBankMarketQueryResponse {
    /// Market index (Bit position on data)
    pub index: u32,
    /// maToken contract address
    pub ma_token_address: Addr,
    /// Indicated whether the asset is native or a cw20 token
    pub asset_type: MarsAssetType,

    /// Max uusd that can be borrowed per uusd collateral when using the asset as collateral
    pub max_loan_to_value: Decimal,
    /// uusd amount in debt position per uusd of asset collateral that if surpassed makes the user's position liquidatable.
    pub liquidation_threshold: Decimal,
    /// Bonus amount of collateral liquidator get when repaying user's debt (Will get collateral
    /// from user in an amount equal to debt repayed + bonus)
    pub liquidation_bonus: Decimal,
    /// Portion of the borrow rate that is kept as protocol rewards
    pub reserve_factor: Decimal,

    /// Interest rate strategy to calculate borrow_rate and liquidity_rate
    pub interest_rate_strategy: InterestRateStrategy,

    /// Borrow index (Used to compute borrow interest)
    pub borrow_index: Decimal,
    /// Liquidity index (Used to compute deposit interest)
    pub liquidity_index: Decimal,
    /// Rate charged to borrowers
    pub borrow_rate: Decimal,
    /// Rate paid to depositors
    pub liquidity_rate: Decimal,
    /// Timestamp (seconds) where indexes and rates where last updated
    pub interests_last_updated: u64,

    /// Total debt scaled for the market's currency
    pub debt_total_scaled: Uint128,

    /// If false cannot do any action (deposit/withdraw/borrow/repay/liquidate)
    pub active: bool,
    /// If false cannot deposit
    pub deposit_enabled: bool,
    /// If false cannot borrow
    pub borrow_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InterestRateStrategy {
    Dynamic(DynamicInterestRate),
    Linear(LinearInterestRate),
}

/// Dynamic interest rate model
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DynamicInterestRate {
    /// Minimum borrow rate
    pub min_borrow_rate: Decimal,
    /// Maximum borrow rate
    pub max_borrow_rate: Decimal,
    /// Proportional parameter for the PID controller
    pub kp_1: Decimal,
    /// Optimal utilization rate targeted by the PID controller. Interest rate will decrease when lower and increase when higher
    pub optimal_utilization_rate: Decimal,
    /// Min error that triggers Kp augmentation
    pub kp_augmentation_threshold: Decimal,
    /// Kp value when error threshold is exceeded
    pub kp_2: Decimal,
}


/// Linear interest rate model
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LinearInterestRate {
    /// Optimal utilization rate
    pub optimal_utilization_rate: Decimal,
    /// Base rate
    pub base: Decimal,
    /// Slope parameter for interest rate model function when utilization_rate < optimal_utilization_rate
    pub slope_1: Decimal,
    /// Slope parameter for interest rate model function when utilization_rate >= optimal_utilization_rate
    pub slope_2: Decimal,
}




/// Represents either a native asset or a cw20. Meant to be used as part of a msg
/// in a contract call and not to be used internally
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarsAsset {
    Cw20 { contract_addr: String },
    Native { denom: String },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarsAssetType {
    Cw20,
    Native,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RedBankAssetsInfo {
    pub asset_info: MarsAsset,
    pub ma_token_address: Addr,
    pub max_loan_to_value: Decimal256,
    pub liquidation_threshold: Decimal256,
    pub liquidation_bonus: Decimal256,
}




