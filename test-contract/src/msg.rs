use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{from_binary, to_binary,  AllBalanceResponse, Api, BalanceResponse, Binary, BankQuery, Coin, CosmosMsg, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, Uint128, WasmMsg, WasmQuery};
use cosmwasm_storage::to_length_prefixed;
use crate::asset::AssetInfo;

use terra_cosmwasm::TerraRoute;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssertMinimumReceive {
    pub asset_info: AssetInfo,
    pub prev_balance: Uint128,
    pub minimum_receive: Uint128,
    pub receiver: HumanAddr,
}

pub fn query_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account_addr: &HumanAddr,
    denom: String,
) -> StdResult<Uint128> {
    // load price form the oracle
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: HumanAddr::from(account_addr),
        denom,
    }))?;
    Ok(balance.amount.amount)
}

pub fn query_all_balances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account_addr: &HumanAddr,
) -> StdResult<Vec<Coin>> {
    // load price form the oracle
    let all_balances: AllBalanceResponse =
        deps.querier
            .query(&QueryRequest::Bank(BankQuery::AllBalances {
                address: HumanAddr::from(account_addr),
            }))?;
    Ok(all_balances.amount)
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

pub fn query_token_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_addr: &HumanAddr,
    account_addr: &HumanAddr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    let res: Binary = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: HumanAddr::from(contract_addr),
            key: Binary::from(concat(
                &to_length_prefixed(b"balance").to_vec(),
                (deps.api.canonical_address(&account_addr)?).as_slice(),
            )),
        }))
        .unwrap_or_else(|_| to_binary(&Uint128::zero()).unwrap());

    from_binary(&res)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Trade { amount: Coin },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
}

// // We define a custom struct for each query response
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CountResponse {
//     pub count: i32,
// }


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WhiteWhaleMsgWrapper {
    TerraMsgWrapper{ route: TerraRoute, msg_data: WhiteWhaleMsg },
    TerraSwapMsgWrapper{ operations: Vec<SwapOperation>, minimum_receive: Option<Uint128>, to: Option<HumanAddr>,},
    TerraSwapMsgMinAssert{ execute: WasmMsg },
}

// this is a helper to be able to return these as CosmosMsg easier
impl Into<CosmosMsg<WhiteWhaleMsgWrapper>> for WhiteWhaleMsgWrapper {
    fn into(self) -> CosmosMsg<WhiteWhaleMsgWrapper> {
        CosmosMsg::Custom(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg2 {
    /// Internal use
    /// Swap all offer tokens to ask token
    ExecuteSwapOperation {
        operation: SwapOperation,
        to: Option<HumanAddr>,
    },
    /// Internal use
    /// Check the swap amount is exceed minimum_receive
    AssertMinimumReceive {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        minimum_receive: Uint128,
        receiver: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapOperation {
    NativeSwap {
        offer_denom: String,
        ask_denom: String,
    },
    TerraSwap {
        offer_asset_info: AssetInfo,
        ask_asset_info: AssetInfo,
    },
}

impl SwapOperation {
    pub fn get_target_asset_info(&self) -> AssetInfo {
        match self {
            SwapOperation::NativeSwap { ask_denom, .. } => AssetInfo::NativeToken {
                denom: ask_denom.clone(),
            },
            SwapOperation::TerraSwap { ask_asset_info, .. } => ask_asset_info.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WhiteWhaleMsg {
    Swap {
        trader: HumanAddr,
        offer_coin: Coin,
        ask_denom: String,
    },
    SwapSend {
        from_address: HumanAddr,
        to_address: HumanAddr,
        offer_coin: Coin,
        ask_denom: String,
    },
    TerraSwap {
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<HumanAddr>,
    },
}

// create_swap_msg returns wrapped swap msg
pub fn create_swap_msg(
    trader: HumanAddr,
    offer_coin: Coin,
    ask_denom: String,
) -> CosmosMsg<WhiteWhaleMsgWrapper> {
    WhiteWhaleMsgWrapper::TerraMsgWrapper {
        route: TerraRoute::Market,
        msg_data: WhiteWhaleMsg::Swap {
            trader,
            offer_coin,
            ask_denom,
        },
    }
    .into()
}

// create_swap_send_msg returns wrapped swap send msg
pub fn create_swap_send_msg(
    from_address: HumanAddr,
    to_address: HumanAddr,
    offer_coin: Coin,
    ask_denom: String,
) -> CosmosMsg<WhiteWhaleMsgWrapper> {
    WhiteWhaleMsgWrapper::TerraMsgWrapper {
        route: TerraRoute::Market,
        msg_data: WhiteWhaleMsg::SwapSend {
            from_address,
            to_address,
            offer_coin,
            ask_denom,
        },
    }
    .into()
}
