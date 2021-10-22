// use cosmwasm_std::{
//     entry_point, from_binary, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg,
//     Decimal, Deps, DepsMut, Env, Fraction, MessageInfo, Reply, ReplyOn, Response, StdError,
//     StdResult, SubMsg, Uint128, WasmMsg,
// };
// use protobuf::Message;

// use terraswap::asset::AssetInfo;

// use white_whale::ust_vault::msg::*;

// const INSTANTIATE_REPLY_ID: u8 = 1u8;

// fn get_test_init_msg() -> InitMsg {
//     InitMsg {
//         anchor_money_market_address: "test_mm".to_string(),
//         aust_address: "test_aust".to_string(),
//         profit_check_address: "test_profit_check".to_string(),
//         community_fund_addr: "community_fund".to_string(),
//         warchest_addr: "warchest".to_string(),
//         asset_info: AssetInfo::NativeToken {
//             denom: "uusd".to_string(),
//         },
//         token_code_id: 0u64,
//         warchest_fee: Decimal::percent(10u64),
//         community_fund_fee: Decimal::permille(5u64),
//         max_community_fund_fee: Uint128::from(1000000u64),
//         stable_cap: Uint128::from(100_000_000u64),
//         vault_lp_token_name: None,
//         vault_lp_token_symbol: None,
//         whitelisted_contracts: vec![],
//     }
// }

// #[test]
// fn test_initialization() {
//     let mut deps = mock_dependencies(&[]);

//     let msg = get_test_init_msg();
//     let env = mock_env();
//     let info = MessageInfo {
//         sender: deps.api.addr_validate("creator").unwrap(),
//         funds: vec![],
//     };

//     let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
//     assert_eq!(1, res.messages.len());
// }

// #[test]
// fn test_init_with_non_default_vault_lp_token() {
//     let mut deps = mock_dependencies(&[]);

//     let custom_token_name = String::from("My LP Token");
//     let custom_token_symbol = String::from("MyLP");

//     // Define a custom Init Msg with the custom token info provided
//     let msg = InitMsg {
//         anchor_money_market_address: "test_mm".to_string(),
//         aust_address: "test_aust".to_string(),
//         profit_check_address: "test_profit_check".to_string(),
//         community_fund_addr: "community_fund".to_string(),
//         warchest_addr: "warchest".to_string(),
//         asset_info: AssetInfo::NativeToken {
//             denom: "uusd".to_string(),
//         },
//         token_code_id: 10u64,
//         warchest_fee: Decimal::percent(10u64),
//         community_fund_fee: Decimal::permille(5u64),
//         max_community_fund_fee: Uint128::from(1000000u64),
//         stable_cap: Uint128::from(1000_000_000u64),
//         vault_lp_token_name: Some(custom_token_name.clone()),
//         vault_lp_token_symbol: Some(custom_token_symbol.clone()),
//         whitelisted_contracts: vec![],
//     };

//     // Prepare mock env
//     let env = mock_env();
//     let info = MessageInfo {
//         sender: deps.api.addr_validate("creator").unwrap(),
//         funds: vec![],
//     };

//     let res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
//     // Ensure we have 1 message
//     assert_eq!(1, res.messages.len());
//     // Verify the message is the one we expect but also that our custom provided token name and symbol were taken into account.
//     assert_eq!(
//         res.messages,
//         vec![SubMsg {
//             // Create LP token
//             msg: WasmMsg::Instantiate {
//                 admin: None,
//                 code_id: msg.token_code_id,
//                 msg: to_binary(&TokenInstantiateMsg {
//                     name: custom_token_name.to_string(),
//                     symbol: custom_token_symbol.to_string(),
//                     decimals: 6,
//                     initial_balances: vec![],
//                     mint: Some(MinterResponse {
//                         minter: env.contract.address.to_string(),
//                         cap: None,
//                     }),
//                 })
//                 .unwrap(),
//                 funds: vec![],
//                 label: "".to_string(),
//             }
//             .into(),
//             gas_limit: None,
//             id: u64::from(INSTANTIATE_REPLY_ID),
//             reply_on: ReplyOn::Success,
//         }]
//     );
// }

// #[test]
// fn test_set_ust_cap() {
//     let mut deps = mock_dependencies(&[]);

//     let msg = get_test_init_msg();
//     let env = mock_env();
//     let msg_info = MessageInfo {
//         sender: deps.api.addr_validate("creator").unwrap(),
//         funds: vec![],
//     };

//     let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
//     assert_eq!(1, res.messages.len());

//     let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//     assert_eq!(info.stable_cap, Uint128::from(100_000_000u64));

//     let msg = ExecuteMsg::SetStableCap {
//         stable_cap: Uint128::from(100_000u64),
//     };
//     let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//     let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//     assert_eq!(info.stable_cap, Uint128::from(100_000u64));
// }

// #[test]
// fn test_set_warchest_fee() {
//     let mut deps = mock_dependencies(&[]);

//     let msg = get_test_init_msg();
//     let env = mock_env();
//     let msg_info = MessageInfo {
//         sender: deps.api.addr_validate("creator").unwrap(),
//         funds: vec![],
//     };

//     let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
//     assert_eq!(1, res.messages.len());

//     let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//     assert_eq!(info.stable_cap, Uint128::from(100_000_000u64));

//     let warchest_fee = FEE.load(&deps.storage).unwrap().warchest_fee.share;
//     let new_fee = Decimal::permille(1u64);
//     assert_ne!(warchest_fee, new_fee);
//     let msg = ExecuteMsg::SetFee {
//         community_fund_fee: None,
//         warchest_fee: Some(Fee { share: new_fee }),
//     };
//     let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//     let warchest_fee = FEE.load(&deps.storage).unwrap().warchest_fee.share;
//     assert_eq!(warchest_fee, new_fee);
// }

// #[test]
// fn test_set_community_fund_fee() {
//     let mut deps = mock_dependencies(&[]);

//     let msg = get_test_init_msg();
//     let env = mock_env();
//     let msg_info = MessageInfo {
//         sender: deps.api.addr_validate("creator").unwrap(),
//         funds: vec![],
//     };

//     let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
//     assert_eq!(1, res.messages.len());

//     let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//     assert_eq!(info.stable_cap, Uint128::from(100_000_000u64));

//     let community_fund_fee = FEE
//         .load(&deps.storage)
//         .unwrap()
//         .community_fund_fee
//         .fee
//         .share;
//     let new_fee = Decimal::permille(1u64);
//     let new_max_fee = Uint128::from(42u64);
//     assert_ne!(community_fund_fee, new_fee);
//     let msg = ExecuteMsg::SetFee {
//         community_fund_fee: Some(CappedFee {
//             fee: Fee { share: new_fee },
//             max_fee: new_max_fee,
//         }),
//         warchest_fee: None,
//     };
//     let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//     let community_fund_fee = FEE
//         .load(&deps.storage)
//         .unwrap()
//         .community_fund_fee
//         .fee
//         .share;
//     let community_fund_max_fee = FEE.load(&deps.storage).unwrap().community_fund_fee.max_fee;
//     assert_eq!(community_fund_fee, new_fee);
//     assert_eq!(community_fund_max_fee, new_max_fee);
// }
