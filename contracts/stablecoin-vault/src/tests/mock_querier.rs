#![allow(dead_code)]
#![cfg(test)]

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Api, Binary, Coin, ContractResult, Decimal, Empty,
    OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};

use crate::tests::anchor_mock::mock_epoch_state;
use cosmwasm_storage::to_length_prefixed;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};
use std::collections::HashMap;
use terra_cosmwasm::{
    SwapResponse, TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute,
};
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw, PairInfo, PairInfoRaw};
use terraswap::pair::PoolResponse;
use white_whale::profit_check::msg::LastBalanceResponse;
use white_whale::query::anchor::EpochStateResponse;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    terraswap_pair_querier: TerraswapPairQuerier,
    token_querier: TokenQuerier,
    tax_querier: TaxQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

#[derive(Clone, Default)]
pub struct TerraswapPairQuerier {
    pairs: HashMap<String, PairInfo>,
}

impl TerraswapPairQuerier {
    pub fn new(pairs: &[(&String, &PairInfo)]) -> Self {
        TerraswapPairQuerier {
            pairs: pairs_to_map(pairs),
        }
    }
}

pub(crate) fn pairs_to_map(pairs: &[(&String, &PairInfo)]) -> HashMap<String, PairInfo> {
    let mut pairs_map: HashMap<String, PairInfo> = HashMap::new();
    for (key, pair) in pairs.iter() {
        pairs_map.insert(key.to_string(), (*pair).clone());
    }
    pairs_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            // A custom handler for TerraQueries such as TaxCaps or Rates
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if route == &TerraRoute::Treasury {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax_querier.rate,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self
                                .tax_querier
                                .caps
                                .get(denom)
                                .copied()
                                .unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if route == &TerraRoute::Market {
                    match query_data {
                        TerraQuery::Swap {
                            offer_coin,
                            ask_denom,
                        } => {
                            let res = SwapResponse {
                                receive: Coin {
                                    amount: offer_coin.amount,
                                    denom: String::from(ask_denom),
                                },
                            };

                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            // Manual mocking for smart queries
            // Here we can do alot to mock out messages either by defining a new
            // MockQueryMsg with each call as a type of it
            // Or for more quick multi-contract mocking consider using the contract_addr
            // or directly parsing the message if it is unique
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                // Handle calls for Profit Check; LastBalance
                if contract_addr == &String::from("test_profit_check") {
                    println!("{:?}", request);

                    return SystemResult::Ok(ContractResult::from(to_binary(
                        &LastBalanceResponse {
                            last_balance: Uint128::zero(),
                        },
                    )));
                }
                // Handle calls for Profit Check; LastBalance
                if contract_addr == &String::from("test_mm") {
                    println!("{:?}", request);

                    // Handle Anchor EpochStateQuery
                    if msg == &Binary::from(r#"{"epoch_state":{}}"#.as_bytes()) {
                        return SystemResult::Ok(ContractResult::from(to_binary(
                            &EpochStateResponse {
                                exchange_rate: Decimal256::percent(120),
                                aterra_supply: Uint256::from(1000000u64),
                            },
                        )));
                    }
                    return SystemResult::Ok(ContractResult::from(to_binary(
                        &EpochStateResponse {
                            exchange_rate: Decimal256::percent(120),
                            aterra_supply: Uint256::from(1000000u64),
                        },
                    )));
                }
                // Handle calls for Profit Check; LastBalance
                if contract_addr == &String::from("test_aust") {
                    println!("{:?}", request);

                    // Handle Anchor EpochStateQuery
                    if msg == &Binary::from(r#"{"epoch_state":{}}"#.as_bytes()) {
                        return SystemResult::Ok(ContractResult::from(to_binary(
                            &mock_epoch_state(),
                        )));
                    }

                    return SystemResult::Ok(ContractResult::from(to_binary(
                        &Cw20BalanceResponse {
                            balance: Uint128::zero(),
                        },
                    )));
                }
                // Handle calls for Pair Info
                if contract_addr == &String::from("PAIR0000") {
                    if msg == &Binary::from(r#"{"pool":{}}"#.as_bytes()) {
                        let msg_pool = PoolResponse {
                            assets: [
                                Asset {
                                    amount: Uint128::from(10000u128),
                                    info: AssetInfo::NativeToken {
                                        denom: "whale".to_string(),
                                    },
                                },
                                Asset {
                                    amount: Uint128::from(10000u128),
                                    info: AssetInfo::NativeToken {
                                        denom: "uusd".to_string(),
                                    },
                                },
                            ],
                            total_share: Uint128::from(1000u128),
                        };
                        return SystemResult::Ok(ContractResult::from(to_binary(&msg_pool)));
                    }

                    let msg_balance = PairInfo {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: "whale".to_string(),
                            },
                            AssetInfo::NativeToken {
                                denom: "uusd".to_string(),
                            },
                        ],
                        contract_addr: "PAIR0000".to_string(),
                        liquidity_token: "Liqtoken".to_string(),
                    };

                    return SystemResult::Ok(ContractResult::from(to_binary(&msg_balance)));
                } else {
                    match from_binary(&msg).unwrap() {
                        // AnchorQuery::EpochState{ distributed_interest, aterra_supply} => {

                        //     return SystemResult::Ok(ContractResult::Ok(
                        //         to_binary(&EpochStateResponse{
                        //             exchange_rate: Decimal256::percent(120),
                        //             aterra_supply: Uint256::from(1000000u64)
                        //         }).unwrap()
                        //     ))
                        // }
                        Cw20QueryMsg::Balance { address } => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: format!(
                                                "No balance info exists for the contract {}",
                                                contract_addr
                                            ),
                                            request: msg.as_slice().into(),
                                        })
                                    }
                                };

                            let balance = match balances.get(&address) {
                                Some(v) => *v,
                                None => {
                                    return SystemResult::Ok(ContractResult::Ok(
                                        to_binary(&Cw20BalanceResponse {
                                            balance: Uint128::zero(),
                                        })
                                        .unwrap(),
                                    ));
                                }
                            };

                            SystemResult::Ok(ContractResult::Ok(
                                to_binary(&Cw20BalanceResponse { balance }).unwrap(),
                            ))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                }
            }
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                println!("hello from raw query");
                let key: &[u8] = key.as_slice();
                let prefix_pair_info = to_length_prefixed(b"pair_info").to_vec();

                if key.to_vec() == prefix_pair_info {
                    let pair_info: PairInfo =
                        match self.terraswap_pair_querier.pairs.get(contract_addr) {
                            Some(v) => v.clone(),
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!("PairInfo is not found for {}", contract_addr),
                                    request: key.into(),
                                })
                            }
                        };

                    let api: MockApi = MockApi::default();
                    SystemResult::Ok(ContractResult::from(to_binary(&PairInfoRaw {
                        contract_addr: api
                            .addr_canonicalize(pair_info.contract_addr.as_str())
                            .unwrap(),
                        liquidity_token: api
                            .addr_canonicalize(pair_info.liquidity_token.as_str())
                            .unwrap(),
                        asset_infos: [
                            AssetInfoRaw::NativeToken {
                                denom: "uusd".to_string(),
                            },
                            AssetInfoRaw::NativeToken {
                                denom: "uusd".to_string(),
                            },
                        ],
                    })))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            terraswap_pair_querier: TerraswapPairQuerier::default(),
            token_querier: TokenQuerier::default(),
            tax_querier: TaxQuerier::default(),
        }
    }

    // configure the terraswap pair
    pub fn with_terraswap_pairs(&mut self, pairs: &[(&String, &PairInfo)]) {
        self.terraswap_pair_querier = TerraswapPairQuerier::new(pairs);
    }

    // pub fn with_balance(&mut self, balances: &[(&HumanAddr, &[Coin])]) {
    //     for (addr, balance) in balances {
    //         self.base.update_balance(addr, balance.to_vec());
    //     }
    // }
}
