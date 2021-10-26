
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Deps, Addr, Uint128};
use std::fmt;

use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, Decimal, MessageInfo, QuerierWrapper, StdError,
    StdResult, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terra_cosmwasm::TerraQuerier;
use crate::helper::build_send_cw20_token_msg;
use crate::tax::{deduct_tax, compute_tax };


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AstroportExecuteMsg {
    /// Execute multiple BuyOperation
    ExecuteSwapOperations {
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<Addr>,
    },

    /// Internal use
    /// Swap all offer tokens to ask token
    ExecuteSwapOperation {
        operation: SwapOperation,
        to: Option<String>,
    },
    /// Internal use
    /// Check the swap amount is exceed minimum_receive
    AssertMinimumReceive {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        minimum_receive: Uint128,
        receiver: String,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AstroportCw20HookMsg {
    ExecuteSwapOperations {
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<String>,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AstroportQueryMsg {
    Config {},
    SimulateSwapOperations {
        offer_amount: Uint128,
        operations: Vec<SwapOperation>,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapOperation {
    NativeSwap {
        offer_denom: String,
        ask_denom: String,
    },
    AstroSwap {
        offer_asset_info: AssetInfo,
        ask_asset_info: AssetInfo,
    },
}




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}

static DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

impl Asset {
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    pub fn compute_tax(&self, querier: &QuerierWrapper) -> StdResult<Uint128> {
        let amount = self.amount;
        if let AssetInfo::NativeToken { denom } = &self.info {
            if denom == "uluna" {
                Ok(Uint128::zero())
            } else {
                let terra_querier = TerraQuerier::new(querier);
                let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
                let tax_cap: Uint128 = (terra_querier.query_tax_cap(denom.to_string())?).cap;
                Ok(std::cmp::min(
                    (amount.checked_sub(amount.multiply_ratio(
                        DECIMAL_FRACTION,
                        DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
                    )))?,
                    tax_cap,
                ))
            }
        } else {
            Ok(Uint128::zero())
        }
    }

    pub fn deduct_tax(&self, querier: &QuerierWrapper) -> StdResult<Coin> {
        let amount = self.amount;
        if let AssetInfo::NativeToken { denom } = &self.info {
            Ok(Coin {
                denom: denom.to_string(),
                amount: amount.checked_sub(self.compute_tax(querier)?)?,
            })
        } else {
            Err(StdError::generic_err("cannot deduct tax from token asset"))
        }
    }

    pub fn into_msg(self, querier: &QuerierWrapper, recipient: Addr) -> StdResult<CosmosMsg> {
        let amount = self.amount;

        match &self.info {
            AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: recipient.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
            AssetInfo::NativeToken { .. } => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![self.deduct_tax(querier)?],
            })),
        }
    }

    pub fn assert_sent_native_token_balance(&self, message_info: &MessageInfo) -> StdResult<()> {
        if let AssetInfo::NativeToken { denom } = &self.info {
            match message_info.funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if self.amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
                None => {
                    if self.amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    Token { contract_addr: Addr },
    NativeToken { denom: String },
}

impl fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetInfo::NativeToken { denom } => write!(f, "{}", denom),
            AssetInfo::Token { contract_addr } => write!(f, "{}", contract_addr),
        }
    }
}

impl AssetInfo {
    pub fn is_native_token(&self) -> bool {
        match self {
            AssetInfo::NativeToken { .. } => true,
            AssetInfo::Token { .. } => false,
        }
    }

    pub fn equal(&self, asset: &AssetInfo) -> bool {
        match self {
            AssetInfo::Token { contract_addr, .. } => {
                let self_contract_addr = contract_addr;
                match asset {
                    AssetInfo::Token { contract_addr, .. } => self_contract_addr == contract_addr,
                    AssetInfo::NativeToken { .. } => false,
                }
            }
            AssetInfo::NativeToken { denom, .. } => {
                let self_denom = denom;
                match asset {
                    AssetInfo::Token { .. } => false,
                    AssetInfo::NativeToken { denom, .. } => self_denom == denom,
                }
            }
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AssetInfo::NativeToken { denom } => denom.as_bytes(),
            AssetInfo::Token { contract_addr } => contract_addr.as_bytes(),
        }
    }
}


/// @dev Helper function. Returns a CosmosMsg struct to trade a cw20 asset for a native asset via Astroport router
/// @param astroport_router : Astroport router contract address
/// @param cw20_token_addr : CW20 token contract address which is to be sold
/// @param sell_amount : Number of cw20 tokens to be traded 
/// @param native_denom : Native token to receive against the trade
pub fn trade_cw20_for_native_on_astroport( astroport_router: String, cw20_token_addr: Addr, sell_amount: Uint128, native_denom: String ) -> StdResult<CosmosMsg> {
    let swap_operation = SwapOperation::AstroSwap {
            offer_asset_info: AssetInfo::Token { contract_addr: cw20_token_addr.clone() },
            ask_asset_info: AssetInfo::NativeToken { denom: native_denom }
    };

    let operations_binary = to_binary(&AstroportExecuteMsg::ExecuteSwapOperations {
        operations: vec![swap_operation],
        minimum_receive: None,
        to: None,
    })?;

    let swap_operation_cosmos_msg = build_send_cw20_token_msg( astroport_router, cw20_token_addr.to_string(), sell_amount, operations_binary )?;
    Ok(swap_operation_cosmos_msg)
}

/// @dev Helper function. Returns a CosmosMsg struct to trade a native asset for a cw20 asset via Astroport router
/// @param astroport_router : Astroport router contract address
/// @param native_denom : Native token which is to be sold 
/// @param sell_amount : Number of native tokens to be traded 
/// @param cw20_token_addr : CW20 token contract address to receive against the trade
pub fn trade_native_for_cw20_on_astroport( deps: Deps, astroport_router: String, native_denom: String , sell_amount: Uint128 , cw20_token_addr: Addr ) -> StdResult<CosmosMsg> {
    let swap_operation = SwapOperation::AstroSwap {
            offer_asset_info: AssetInfo::NativeToken { denom: native_denom.clone() }  ,
            ask_asset_info: AssetInfo::Token { contract_addr: cw20_token_addr }
    };

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: astroport_router,
        msg: to_binary(&AstroportExecuteMsg::ExecuteSwapOperations {
            operations: vec![swap_operation],
            minimum_receive: None,
            to: None,
        })?,
        funds: vec![deduct_tax(
            deps,
            Coin {
                denom: native_denom.to_string(),
                amount: sell_amount.into(),
            },
        )?],
    })) 
}


/// @dev Helper function. Returns a CosmosMsg struct to trade a native asset for another native asset via Astroport router
/// @param astroport_router : Astroport router contract address
/// @param sell_denom : Native token which is to be sold 
/// @param sell_amount : Number of native tokens to be traded 
/// @param ask_denom : Native token to receive against the trade
pub fn trade_native_for_native_on_astroport( deps: Deps, astroport_router: String, sell_denom: String , sell_amount: Uint128 , ask_denom: String ) -> StdResult<CosmosMsg> {
    let swap_operation = SwapOperation::NativeSwap {
            offer_denom:  sell_denom.clone()  ,
            ask_denom:  ask_denom.clone() 
    };
    
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:astroport_router,
        funds: vec![deduct_tax(  deps,  Coin {   denom: sell_denom.to_string(),  amount: sell_amount.into(),}, )? ],
        msg: to_binary(&AstroportExecuteMsg::ExecuteSwapOperations {
            operations: vec![swap_operation],
            minimum_receive: None,
            to: None,
        })?,
    }))

}





