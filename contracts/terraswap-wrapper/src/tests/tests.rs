use crate::contract::{execute, instantiate, query};
use crate::mock_querier::mock_dependencies;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, Decimal, Uint128};
use terraswap::asset::{Asset, AssetInfo};

fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        terraswap_pool_addr: "PAIR0000".to_string(),
        trader: "trader".to_string(),
        max_deposit: Asset {
            info: AssetInfo::NativeToken {
                denom: "whale".to_string(),
            },
            amount: Uint128::from(10000000u128),
        },
        min_profit: Asset {
            info: AssetInfo::NativeToken {
                denom: "whale".to_string(),
            },
            amount: Uint128::from(100000u128),
        },
        slippage: Decimal::one(),
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = init_msg();
    let info = mock_info("addr1", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    // TODO: Review in mock querier
    assert_eq!("000000000000000000000000523000000000000000000000005030300000000000000000000000413000000000000000000000000049".to_string(), config.terraswap_pool_addr);
}

#[test]
fn test_set_trader() {
    let mut deps = mock_dependencies(&[]);

    let msg = init_msg();
    let info = mock_info("addr1", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let trader_msg = ExecuteMsg::SetTrader {
        trader: "trader".to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, trader_msg).unwrap();
}

#[test]
fn test_deposit() {
    let mut deps = mock_dependencies(&[]);

    let msg = init_msg();
    let info = mock_info("trader", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let deposit_msg = ExecuteMsg::Deposit {
        funds: vec![
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
    };
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();
}

#[test]
fn test_deposit_fails_for_non_trader_account() {
    let mut deps = mock_dependencies(&[]);

    let msg = init_msg();
    let info = mock_info("nonthetrader", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let deposit_msg = ExecuteMsg::Deposit {
        funds: vec![
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
    };
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap_err();
}

#[test]
fn test_withdraw_fails_for_non_trader() {
    let mut deps = mock_dependencies(&[]);

    let msg = init_msg();
    let info = mock_info("nonthetrader", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let withdraw_msg = ExecuteMsg::Withdraw {
        funds: vec![
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
    };
    let _res = execute(deps.as_mut(), mock_env(), info, withdraw_msg).unwrap_err();
}

#[test]
fn test_withdraw() {
    let mut deps = mock_dependencies(&[]);

    let msg = init_msg();
    let info = mock_info("trader", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let withdraw_msg = ExecuteMsg::Withdraw {
        funds: vec![
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
    };
    let res = execute(deps.as_mut(), mock_env(), info, withdraw_msg).unwrap();
    assert_eq!(1, res.messages.len());
}

// #[test]
// fn test_spend() {
//     let mut deps = mock_dependencies(&[]);
//     let msg = init_msg();
//     let info = mock_info("addr0000", &[]);

//     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//     let msg = ExecuteMsg::Spend {
//         recipient: "addr0000".to_string(),
//         amount: Asset{
//             amount: Uint128::from(1000000u128),
//             info: AssetInfo::NativeToken {
//                 denom: "whale".to_string(),
//             },
//         }
//     };

//     let info = mock_info("addr0000", &[]);
//     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//     assert_eq!(
//         res.messages,
//         vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: "whale".to_string(),
//             funds: vec![],
//             msg: to_binary(&Cw20ExecuteMsg::Transfer {
//                 recipient: "addr0000".to_string(),
//                 amount: Uint128::from(1000000u128),
//             })
//             .unwrap(),
//         }))]
//     );
// }
