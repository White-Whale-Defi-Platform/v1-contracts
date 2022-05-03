use std::fmt;

use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Decimal, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terra_rust_script_derive::CosmWasmContract;
use terraswap::asset::{Asset, AssetInfo};

use crate::fee::{Fee, VaultFee};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
pub struct InstantiateMsg {
    pub bluna_address: String,
    pub cluna_address: String,
    /// The address of the liquidity pool to provide bLuna-Luna assets to for passive income
    pub astro_lp_address: String,
    /// The address of the Astroport factory
    pub astro_factory_address: String,
    pub treasury_addr: String,
    pub memory_addr: String,
    pub asset_info: AssetInfo,
    pub token_code_id: u64,
    pub treasury_fee: Decimal,
    pub flash_loan_fee: Decimal,
    pub commission_fee: Decimal,
    pub vault_lp_token_name: Option<String>,
    pub vault_lp_token_symbol: Option<String>,
    pub unbonding_period: u64,
    pub unbond_handler_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Receive hook for the liquidity token
    Receive(Cw20ReceiveMsg),
    /// Provide liquidity to the vault
    ProvideLiquidity { asset: Asset },
    /// Send back unbonded luna to the user
    WithdrawUnbonded {},
    /// Sets the withdraw fee and flash loan fee
    SetFee {
        flash_loan_fee: Option<Fee>,
        treasury_fee: Option<Fee>,
        commission_fee: Option<Fee>,
    },
    /// Set the admin of the contract
    SetAdmin { admin: String },
    /// Add provided contract to the whitelisted contracts
    AddToWhitelist { contract_addr: String },
    /// Remove provided contract from the whitelisted contracts
    RemoveFromWhitelist { contract_addr: String },
    /// Update the internal State struct
    UpdateState {
        bluna_address: Option<String>,
        astro_lp_address: Option<String>,
        memory_address: Option<String>,
        whitelisted_contracts: Option<Vec<String>>,
        allow_non_whitelisted: Option<bool>,
        unbonding_period: Option<u64>,
    },
    /// Execute a flashloan
    FlashLoan { payload: FlashLoanPayload },
    /// Swaps the passive strategy token rewards for luna
    SwapRewards {},
    /// Internal callback message
    Callback(CallbackMsg),
    /// Messages sent by unbond handlers to the vault
    UnbondHandler(UnbondHandlerMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FlashLoanPayload {
    pub requested_asset: Asset,
    pub callback: Binary,
}

/// MigrateMsg allows a privileged contract administrator to run
/// a migration on the contract. In this case it is just migrating
/// from one terra code to the same code, but taking advantage of the
/// migration step to set a new validator.
///
/// Note that the contract doesn't enforce permissions here, this is done
/// by blockchain logic (in the future by blockchain governance)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    AfterTrade { loan_fee: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnbondHandlerMsg {
    AfterUnbondHandlerReleased {
        unbond_handler_addr: String,
        previous_owner: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnbondActionReply {
    /// Message send right after creating a new unbond handler instance
    /// Used to trigger a reply and execute the unbond message on the handler
    Unbond { bluna_amount: Uint128 },
}

// Modified from
// https://github.com/CosmWasm/cosmwasm-plus/blob/v0.2.3/packages/cw20/src/receiver.rs#L15
impl CallbackMsg {
    pub fn to_cosmos_msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
        &self,
        contract_addr: &Addr,
    ) -> StdResult<CosmosMsg<T>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&ExecuteMsg::Callback(self.clone()))?,
            funds: vec![],
        }))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum VaultQueryMsg {
    PoolConfig {},
    PoolState {},
    State {},
    Fees {},
    EstimateWithdrawFee {
        amount: Uint128,
    },
    VaultValue {},
    LastBalance {},
    LastProfit {},
    WithdrawableUnbonded {
        address: String,
    },
    UnbondRequests {
        address: String,
        start_from: Option<u64>,
        limit: Option<u32>,
    },
    AllHistory {
        start_from: Option<u64>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Unbond {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 4],
    pub total_value_in_luna: Uint128,
    pub total_share: Uint128,
    pub liquidity_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositResponse {
    pub deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValueResponse {
    pub total_luna_value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeResponse {
    pub fees: VaultFee,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EstimateDepositFeeResponse {
    pub fee: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EstimateWithdrawFeeResponse {
    pub fee: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub anchor_money_market_address: String,
    pub aust_address: String,
    pub allow_non_whitelisted: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastBalanceResponse {
    pub last_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastProfitResponse {
    pub last_profit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondRequestsResponse {
    pub address: String,
    pub requests: UnbondRequestResponse,
}

pub type UnbondRequestResponse = Vec<(u64, Uint128)>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllHistoryResponse {
    pub history: Vec<UnbondHistoryResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondHistoryResponse {
    pub batch_id: u64,
    pub time: u64,
    pub amount: Uint128,
    pub released: bool,
}
