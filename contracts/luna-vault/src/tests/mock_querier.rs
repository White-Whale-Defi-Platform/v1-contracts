#![allow(dead_code)]
#![cfg(test)]

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, Binary, Coin, ContractResult, Decimal,
    OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};

use crate::tests::anchor_mock::mock_epoch_state;
use astroport::asset::{Asset, AssetInfo, PairInfo};
use astroport::factory::PairType;
use astroport::pair::PoolResponse;
use cosmwasm_storage::to_length_prefixed;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};
use std::collections::HashMap;
use std::convert::TryInto;
use terra_cosmwasm::{
    SwapResponse, TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute,
};
use terraswap::asset::{AssetInfoRaw, PairInfoRaw};
use thiserror::private::DisplayAsDisplay;
use white_whale::astroport_helper::SimulationResponse;

use white_whale::query::anchor::{AnchorQuery, EpochStateResponse, UnbondRequestsResponse};
use white_whale::query::anchor::AnchorQuery::UnbondRequests;
use crate::pool_info::PoolInfo as VaultPoolInfo;

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
    astroport_factory_querier: AstroportFactoryQuerier,
}

#[derive(Clone, Default)]
pub struct AstroportFactoryQuerier {
    pairs: HashMap<String, PairInfo>,
}

impl AstroportFactoryQuerier {
    pub fn new(pairs: &[(&String, &PairInfo)]) -> Self {
        AstroportFactoryQuerier {
            pairs: pairs_to_map(pairs),
        }
    }
}

pub(crate) fn pairs_to_map_astro(pairs: &[(&String, &PairInfo)]) -> HashMap<String, PairInfo> {
    let mut pairs_map: HashMap<String, PairInfo> = HashMap::new();
    for (key, pair) in pairs.iter() {
        pairs_map.insert(key.to_string(), (*pair).clone());
    }
    pairs_map
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
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        print!("Request hit the mock querier \n {:?}", request);
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
                if contract_addr == &String::from("test_mm") {

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
                if contract_addr == &String::from("bluna") {

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

                // Handle the memory contract
                // Becuase of another handler in this file, all queried contracts come back with contract_from_memory
                // This is a crude v1 impl for mocking, eventually this should be changed to have multiple
                // Makes it so that ANY functionality within a memory queryied contract that needs to be covered can and should be covered here
                if contract_addr == &String::from("contract_from_memory") {
                    // if msg == &Binary::from(r#"{"unbond_requests":{}}"#.as_bytes()) {
                    return SystemResult::Ok(ContractResult::Ok(
                        to_binary(&UnbondRequestsResponse {
                            address: "".to_string(),
                            requests: vec![],
                        }).unwrap()
                    ));
                    // }
                }


                // Handle calls for Pair Info
                if contract_addr == &String::from("astro") || contract_addr == &String::from("anchor") {
                    print!("Handling call for astro LP token with name {:?}", contract_addr);


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


                    if contract_addr == &String::from("anchor") {
                        let msg_balance = VaultPoolInfo {
                            asset_infos: [
                                terraswap::asset::AssetInfo::NativeToken {
                                    denom: "astro".to_string(),
                                },
                                terraswap::asset::AssetInfo::NativeToken {
                                    denom: "uusd".to_string(),
                                },
                                terraswap::asset::AssetInfo::NativeToken {
                                    denom: "uusd".to_string(),
                                },
                                terraswap::asset::AssetInfo::NativeToken {
                                    denom: "uusd".to_string(),
                                },
                            ],
                            contract_addr: Addr::unchecked("PAIR0000"),
                            liquidity_token: Addr::unchecked("liqtoken"),
                        };
                        return SystemResult::Ok(ContractResult::from(to_binary(&msg_balance)));
                    } else {
                        if msg == &Binary::from(r#"{"pair":{}}"#.as_bytes()) {
                            println!("Were looking for pair");
                            let mut msg_balance = PairInfo {
                                asset_infos: [
                                    AssetInfo::Token {
                                        contract_addr: Addr::unchecked("bluna"),
                                    },
                                    AssetInfo::NativeToken {
                                        denom: "uusd".to_string(),
                                    },
                                ],
                                contract_addr: Addr::unchecked("PAIR0000"),
                                liquidity_token: Addr::unchecked("liqtoken"),
                                pair_type: PairType::Xyk {},
                            };
                            return SystemResult::Ok(ContractResult::from(to_binary(&msg_balance)));

                        }

                        if msg == &Binary::from(r#"{"pool":{}}"#.as_bytes()) {
                            println!("Were looking for pool");
                            let myvac = [
                                Asset {
                                    amount: Uint128::new(1000u128),
                                    info: AssetInfo::Token {
                                        contract_addr: Addr::unchecked("bluna"),
                                    },
                                },
                                Asset {
                                    amount: Uint128::new(1000u128),
                                    info: AssetInfo::NativeToken {
                                        denom: "uusd".to_string(),
                                    },
                                },
                            ];
                            return SystemResult::Ok(ContractResult::from(to_binary(&myvac)));

                        }
                        // By now we are expecting either query pools or simulation

                        // Note this is an interesting one
                        // I know exactly what 'Binary[12,12,12,12]' combo I wanted and was soooo confused as I could not read what the message was
                        // in the end I do as_display and to_string to as least give me something simple to compare too
                        if msg.as_display().to_string() == String::from("eyJzaGFyZSI6eyJhbW91bnQiOiIwIn19") {
                            let myvac = [
                                Asset {
                                    amount: Uint128::new(1000u128),
                                    info: AssetInfo::Token {
                                        contract_addr: Addr::unchecked("bluna"),
                                    },
                                },
                                Asset {
                                    amount: Uint128::new(1000u128),
                                    info: AssetInfo::NativeToken {
                                        denom: "uusd".to_string(),
                                    },
                                },
                            ];
                            return SystemResult::Ok(ContractResult::from(to_binary(&myvac)));
                        }

                        // Only Simulation or a query I have not encountered/mocked in this contract left
                        let msg: SimulationResponse = SimulationResponse {
                            return_amount: Default::default(),
                            spread_amount: Default::default(),
                            commission_amount: Default::default(),
                        };
                        return SystemResult::Ok(ContractResult::from(to_binary(&msg)));
                    }
                } else {
                    match from_binary(msg).unwrap() {
                        Cw20QueryMsg::Balance { address } => {
                            // Handle Calls for the liquidity token, in our case we mostly only need balances
                            // No liquidity token is actually created in mock land so we instead mock its behaviours here
                            println!("{:?}", contract_addr);
                            if contract_addr == &String::from("liqtoken") || contract_addr == &String::from("cluna") {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_binary(&Cw20BalanceResponse { balance: Uint128::zero() }).unwrap(),
                                ));
                            }


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
                                        });
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

                if contract_addr == &String::from("memory") {
                    return SystemResult::Ok(ContractResult::Ok(
                        to_binary(&"contract_from_memory".to_string()).unwrap(),
                    ));
                }


                if key.to_vec() == prefix_pair_info {
                    let pair_info: PairInfo =
                        match self.terraswap_pair_querier.pairs.get(contract_addr) {
                            Some(v) => v.clone(),
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!("PairInfo is not found for {}", contract_addr),
                                    request: key.into(),
                                });
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
            astroport_factory_querier: AstroportFactoryQuerier::default(),
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

    // Configure the Astroport pair
    pub fn with_astroport_pairs(&mut self, pairs: &[(&String, &PairInfo)]) {
        self.astroport_factory_querier = AstroportFactoryQuerier::new(pairs);
    }
}
