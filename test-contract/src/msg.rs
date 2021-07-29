use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{from_binary, to_binary,  AllBalanceResponse, Api, BalanceResponse, Binary, BankQuery, Coin, Decimal, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, Uint128, WasmQuery};
use cosmwasm_storage::to_length_prefixed;
use cw20::Cw20ReceiveMsg;

use crate::pair::{HandleMsg as PairMsg};
use crate::asset::{Asset, AssetInfo};

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
pub struct InitMsg {
    pub pool_address: HumanAddr,
    pub asset_info: AssetInfo,
    /// Token contract code id for initialization
    pub token_code_id: u64
    // Hook for post initalization
    // pub init_hook: Option<InitHook>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMsg {
    DepositStable{},
    RedeemStable{},
    Send{ contract: HumanAddr, amount: Uint128, msg: Binary }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Cw20ReceiveMsg),
    AbovePeg { amount: Coin, luna_price: Coin, residual_luna: Uint128 },
    BelowPeg { amount: Coin, luna_price: Coin, residual_luna: Uint128 },
    PostInitialize{},
    ProvideLiquidity {
        asset: Asset
    },
    AnchorDeposit{ amount: Coin },
    AnchorWithdraw{ amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Asset{},
    Pool{}
    // GetCount returns the current count as a json-encoded number
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub asset: Asset,
    pub total_share: Uint128,
}

pub fn create_terraswap_msg(
    offer: Coin,
    belief_price: Decimal
) -> PairMsg {
    let offer = Asset{
        info: AssetInfo::NativeToken{ denom: offer.denom.clone() },
        amount: offer.amount
    };
    PairMsg::Swap{
        offer_asset: offer,
        belief_price: Some(belief_price),
        max_spread: Some(Decimal::from_ratio(Uint128(1), Uint128(100))),
        to: None,
    }
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
        offer_coin: offer_coin,
        ask_denom: ask_denom,
        minimum_receive: minimum_receive * Decimal::from_ratio(Uint128(99), Uint128(100))
    }
}
