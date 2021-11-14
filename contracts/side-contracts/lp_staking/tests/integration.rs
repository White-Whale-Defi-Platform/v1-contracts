// use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
// use cosmwasm_std::{attr, Addr, Timestamp, Uint128};
// use cw_multi_test::{App, BankKeeper, ContractWrapper, Executor};
// use white_whale::lp_staking::ExecuteMsg::UpdateConfig;
// use cosmwasm_bignumber::{Decimal256, Uint256};

// use white_whale::lp_staking::{
//     ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
//     StakerInfoResponse, StateResponse, TimeResponse, UpdateConfigMsg,
// };

// fn mock_app() -> App {
//     let api = MockApi::default();
//     let env = mock_env();
//     let bank = BankKeeper::new();
//     let storage = MockStorage::new();
//     // let tmq = TerraMockQuerier::new(MockQuerier::new(&[]));

//     App::new(api, env.block, bank, storage)
// }

// fn init_contracts(app: &mut App) -> (Addr, Addr, InstantiateMsg) {
//     let owner = Addr::unchecked("contract_owner");

//     // Instantiate WHALE Token Contract
//     let cw20_token_contract = Box::new(ContractWrapper::new(
//         cw20_base::contract::execute,
//         cw20_base::contract::instantiate,
//         cw20_base::contract::query,
//     ));

//     let cw20_token_code_id = app.store_code(cw20_token_contract);

//     let msg = cw20_base::msg::InstantiateMsg {
//         name: String::from("Whale token"),
//         symbol: String::from("WHALE"),
//         decimals: 6,
//         initial_balances: vec![],
//         mint: Some(cw20::MinterResponse {
//             minter: owner.to_string(),
//             cap: None,
//         }),
//         marketing: None,
//     };

//     let whale_token_instance = app
//         .instantiate_contract(
//             cw20_token_code_id,
//             owner.clone(),
//             &msg,
//             &[],
//             String::from("WHALE"),
//             None,
//         )
//         .unwrap();

//     // Instantiate LP Token Contract
//     let msg = cw20_base::msg::InstantiateMsg {
//         name: String::from("Astro LP"),
//         symbol: String::from("uLP"),
//         decimals: 6,
//         initial_balances: vec![],
//         mint: Some(cw20::MinterResponse {
//             minter: owner.to_string(),
//             cap: None,
//         }),
//         marketing: None,
//     };

//     let lp_token_instance = app
//         .instantiate_contract(
//             cw20_token_code_id,
//             owner.clone(),
//             &msg,
//             &[],
//             String::from("WHALE"),
//             None,
//         )
//         .unwrap();

//     // Instantiate Staking Contract
//     let staking_contract = Box::new(ContractWrapper::new(
//         whale_lp_staking::contract::execute,
//         whale_lp_staking::contract::instantiate,
//         whale_lp_staking::contract::query,
//     ));

//     let staking_code_id = app.store_code(staking_contract);

//     let init_timestamp = 1571797421;
//     let till_timestamp = 1_000_000_00000;
//     let reward_increase = Decimal256::from_ratio(2u64, 100u64);

//     let staking_instantiate_msg = InstantiateMsg {
//         owner: Some(owner.to_string()),
//         whale_token: Some("whale_token".to_string()),
//         staking_token: Some("staking_token".to_string()),
//         init_timestamp: init_timestamp,
//         till_timestamp: till_timestamp,
//         cycle_rewards: Some(Uint256::from(100000000u64)),
//         cycle_duration: 10u64,
//         reward_increase: Some(reward_increase),
//     };

//     // Init contract
//     let staking_instance = app
//         .instantiate_contract(
//             staking_code_id,
//             owner.clone(),
//             &staking_instantiate_msg,
//             &[],
//             "airdrop",
//             None,
//         )
//         .unwrap();

//     (
//         staking_instance,
//         whale_token_instance,
//         staking_instantiate_msg,
//     )
// }

// fn mint_some_whale(
//     app: &mut App,
//     owner: Addr,
//     whale_token_instance: Addr,
//     amount: Uint128,
//     to: String,
// ) {
//     let msg = cw20::Cw20ExecuteMsg::Mint {
//         recipient: to.clone(),
//         amount: amount,
//     };
//     let res = app
//         .execute_contract(owner.clone(), whale_token_instance.clone(), &msg, &[])
//         .unwrap();
//     assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
//     assert_eq!(res.events[1].attributes[2], attr("to", to));
//     assert_eq!(res.events[1].attributes[3], attr("amount", amount));
// }

// #[test]
// fn proper_initialization() {
//     let mut app = mock_app();
//     let (staking_instance, _, init_msg) = init_contracts(&mut app);

//     let resp: ConfigResponse = app
//         .wrap()
//         .query_wasm_smart(&staking_instance, &QueryMsg::Config {})
//         .unwrap();

//     // Check config
//     assert_eq!(init_msg.whale_token.unwrap(), resp.whale_token);
//     assert_eq!(
//         init_msg.staking_token.unwrap(),
//         resp.staking_token
//     );
//     assert_eq!(init_msg.owner.unwrap(), resp.owner);
//     assert_eq!(init_msg.init_timestamp, resp.init_timestamp);
//     assert_eq!(init_msg.till_timestamp, resp.till_timestamp);
//     assert_eq!(init_msg.cycle_duration, resp.cycle_duration);
//     assert_eq!(init_msg.reward_increase.unwrap(), resp.reward_increase);

//     // Check state
//     let resp: StateResponse = app
//         .wrap()
//         .query_wasm_smart(&staking_instance, &QueryMsg::State { timestamp: None })
//         .unwrap();

//     assert_eq!(0u64, resp.current_cycle);
//     assert_eq!(init_msg.cycle_rewards.unwrap(), resp.current_cycle_rewards);
//     assert_eq!(Uint256::zero(), resp.total_bond_amount);
//     assert_eq!(Decimal256::zero(), resp.global_reward_index);
// }

// #[test]
// fn test_update_config() {
//         let mut app = mock_app();
//         let (staking_instance, _, init_msg) = init_contracts(&mut app);

//         // *** Test : Error "Only owner can update configuration" ****

//         let mut new_config_msg = UpdateConfigMsg {
//             owner: Some("new_owner".to_string()),
//             staking_token: Some("new_staking_token".to_string()),
//             reward_increase: Some(Decimal256::from_ratio(12u64,1u64)),
//         };

//         let mut update_config_msg = UpdateConfig {
//             new_config: new_config_msg.clone(),
//         };

//         let err = app
//             .execute_contract(
//                 Addr::unchecked("wrong_owner"),
//                 staking_instance.clone(),
//                 &update_config_msg,
//                 &[],
//             )
//             .unwrap_err();

//         assert_eq!(
//             err.to_string(),
//             "Generic error: Only owner can update configuration"
//         );

//         // *** Test : Error "Invalid reward increase ratio" ****

//         let mut new_config_msg = UpdateConfigMsg {
//             owner: Some("new_owner".to_string()),
//             staking_token: Some("new_staking_token".to_string()),
//             reward_increase: Some(Decimal256::from_ratio(12u64,1u64)),
//         };

//         let mut update_config_msg = UpdateConfig {
//             new_config: new_config_msg.clone(),
//         };

//         let err = app
//             .execute_contract(
//                 Addr::unchecked(init_msg.owner.clone().unwrap()),
//                 staking_instance.clone(),
//                 &update_config_msg,
//                 &[],
//             )
//             .unwrap_err();

//         assert_eq!(
//             err.to_string(),
//             "Generic error: Invalid reward increase ratio"
//         );

//         // *** Test : Should update successfully ****
//         new_config_msg.reward_increase = Some(Decimal256::from_ratio(1u64,12u64));
//         update_config_msg = UpdateConfig {
//             new_config: new_config_msg.clone(),
//         };

//         // should be a success
//         app.execute_contract(
//             Addr::unchecked(init_msg.owner.clone().unwrap()),
//             staking_instance.clone(),
//             &update_config_msg,
//             &[],
//         )
//         .unwrap();

//         let resp: ConfigResponse = app
//             .wrap()
//             .query_wasm_smart(&staking_instance, &QueryMsg::Config {})
//             .unwrap();

//         // Check config and make sure all fields are updated
//         assert_eq!(init_msg.whale_token.unwrap(), resp.whale_token);
//         assert_eq!(
//             new_config_msg.staking_token.unwrap(),
//             resp.staking_token
//         );
//         assert_eq!(new_config_msg.owner.unwrap(), resp.owner);
//         assert_eq!(init_msg.init_timestamp, resp.init_timestamp);
//         assert_eq!(init_msg.till_timestamp, resp.till_timestamp);
//         assert_eq!(init_msg.cycle_duration, resp.cycle_duration);
//         assert_eq!(new_config_msg.reward_increase.unwrap(), resp.reward_increase);

//     }

//     #[test]
//     fn test_bond_tokens() {
//         let mut app = mock_app();
//         let (staking_instance, _, init_msg) = init_contracts(&mut app);

//         // ***
//         // *** Test :: Staking before reward distribution goes live ***
//         // ***

// //         let mut env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_03),
// //             ..Default::default()
// //         });

// //         let amount_to_stake = 1000u128;
// //         let mut msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
// //             msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
// //             sender: "depositor".to_string(),
// //             amount: Uint128::new(amount_to_stake.clone()),
// //         });
// //         let mut bond_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "1000"),
// //                 attr("total_bonded", "1000"),
// //             ]
// //         );
// //         // Check Global State
// //         let mut state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(0u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(100u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_03, state_.last_distributed);
// //         assert_eq!(Uint256::from(1000u64), state_.total_bond_amount);
// //         assert_eq!(Decimal256::zero(), state_.global_reward_index);
// //         // Check User State
// //         let mut user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(1000u64), user_position_.bond_amount);
// //         assert_eq!(Decimal256::zero(), user_position_.reward_index);
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: Staking when reward distribution goes live ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_13),
// //             ..Default::default()
// //         });
// //         bond_res_s = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "1000"),
// //                 attr("total_bonded", "2000"),
// //             ]
// //         );
// //         // Check Global State
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(0u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(100u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_13, state_.last_distributed);
// //         assert_eq!(Uint256::from(2000u64), state_.total_bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(30u64, 1000u64),
// //             state_.global_reward_index
// //         );
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(2000u64), user_position_.bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(30u64, 1000u64),
// //             user_position_.reward_index
// //         );
// //         assert_eq!(Uint256::from(30u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: Staking when reward distribution is live (within a block) ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_19),
// //             ..Default::default()
// //         });
// //         msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
// //             msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
// //             sender: "depositor".to_string(),
// //             amount: Uint128::new(10u128),
// //         });
// //         bond_res_s = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "10"),
// //                 attr("total_bonded", "2010"),
// //             ]
// //         );
// //         // Check Global State
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(0u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(100u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_19, state_.last_distributed);
// //         assert_eq!(Uint256::from(2010u64), state_.total_bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(60u64, 1000u64),
// //             state_.global_reward_index
// //         );
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(2010u64), user_position_.bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(60u64, 1000u64),
// //             user_position_.reward_index
// //         );
// //         assert_eq!(Uint256::from(90u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: Staking when reward distribution is live (spans multiple blocks) ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_47),
// //             ..Default::default()
// //         });
// //         msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
// //             msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
// //             sender: "depositor".to_string(),
// //             amount: Uint128::new(70u128),
// //         });
// //         bond_res_s = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "70"),
// //                 attr("total_bonded", "2080"),
// //             ]
// //         );
// //         // Check Global State
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(3u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(109u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_47, state_.last_distributed);
// //         assert_eq!(Uint256::from(2080u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(2080u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(385u64), user_position_.pending_reward);

// //         // Test :: Staking after reward distribution is over

// //         // ***
// //         // *** Test :: Staking when reward distribution is about to be over ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_001_15),
// //             ..Default::default()
// //         });
// //         msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
// //             msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
// //             sender: "depositor".to_string(),
// //             amount: Uint128::new(70u128),
// //         });
// //         bond_res_s = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "70"),
// //                 attr("total_bonded", "2150"),
// //             ]
// //         );
// //         // Check Global State
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(10u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(0u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_001_10, state_.last_distributed);
// //         assert_eq!(Uint256::from(2150u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(2150u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(1135u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: Staking when reward distribution is over ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_001_31),
// //             ..Default::default()
// //         });
// //         msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
// //             msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
// //             sender: "depositor".to_string(),
// //             amount: Uint128::new(30u128),
// //         });
// //         bond_res_s = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "30"),
// //                 attr("total_bonded", "2180"),
// //             ]
// //         );
// //         // Check Global State
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(10u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(0u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_001_10, state_.last_distributed);
// //         assert_eq!(Uint256::from(2180u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(2180u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(1135u64), user_position_.pending_reward);
//     }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;

// //     #[test]
// //     fn test_unbond_tokens() {
// //         let mut info = mock_info("staking_token");
// //         let mut deps = th_setup(&[]);

// //         // ***
// //         // *** Test :: Staking when reward distribution is live (within a block) ***
// //         // ***

// //         let mut env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_15),
// //             ..Default::default()
// //         });
// //         let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
// //             msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
// //             sender: "depositor".to_string(),
// //             amount: Uint128::new(10000000u128),
// //         });
// //         let bond_res_s = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "10000000"),
// //                 attr("total_bonded", "10000000"),
// //             ]
// //         );
// //         // Check Global State
// //         let mut state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(0u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(100u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_15, state_.last_distributed);
// //         assert_eq!(Uint256::from(10000000u64), state_.total_bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(0u64, 1000u64),
// //             state_.global_reward_index
// //         );
// //         // Check User State
// //         let mut user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(10000000u64), user_position_.bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(0u64, 1000u64),
// //             user_position_.reward_index
// //         );
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: "Cannot unbond more than bond amount" Error ***
// //         // ***
// //         info = mock_info("depositor");
// //         let mut unbond_msg = ExecuteMsg::Unbond {
// //             amount: Uint256::from(10000001u64),
// //             withdraw_pending_reward: Some(false),
// //         };
// //         let unbond_res_f = execute(deps.as_mut(), env.clone(), info.clone(), unbond_msg.clone());
// //         assert_generic_error_message(unbond_res_f, "Cannot unbond more than bond amount");

// //         // ***
// //         // *** Test :: UN-Staking when reward distribution is live & don't claim rewards (same block) ***
// //         // ***
// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_17),
// //             ..Default::default()
// //         });
// //         unbond_msg = ExecuteMsg::Unbond {
// //             amount: Uint256::from(100u64),
// //             withdraw_pending_reward: Some(false),
// //         };
// //         let unbond_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), unbond_msg.clone()).unwrap();
// //         assert_eq!(
// //             unbond_res_s.messages,
// //             vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                 contract_addr: "staking_token".to_string(),
// //                 msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                     recipient: "depositor".to_string(),
// //                     amount: Uint128::from(100u64),
// //                 })
// //                 .unwrap(),
// //                 funds: vec![]
// //             }))]
// //         );
// //         assert_eq!(
// //             unbond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Unbond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "100"),
// //                 attr("total_bonded", "9999900"),
// //                 attr("claimed_rewards", "0"),
// //             ]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(0u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(100u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_17, state_.last_distributed);
// //         assert_eq!(Uint256::from(9999900u64), state_.total_bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(20u64, 10000000u64),
// //             state_.global_reward_index
// //         );
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(9999900u64), user_position_.bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(20u64, 10000000u64),
// //             user_position_.reward_index
// //         );
// //         assert_eq!(Uint256::from(20u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: UN-Staking when reward distribution is live & claim rewards (same block) ***
// //         // ***
// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_19),
// //             ..Default::default()
// //         });
// //         unbond_msg = ExecuteMsg::Unbond {
// //             amount: Uint256::from(100u64),
// //             withdraw_pending_reward: Some(true),
// //         };
// //         let unbond_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), unbond_msg.clone()).unwrap();
// //         assert_eq!(
// //             unbond_res_s.messages,
// //             vec![
// //                 SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                     contract_addr: "whale_token".to_string(),
// //                     msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                         recipient: "depositor".to_string(),
// //                         amount: Uint128::from(40u64),
// //                     })
// //                     .unwrap(),
// //                     funds: vec![]
// //                 })),
// //                 SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                     contract_addr: "staking_token".to_string(),
// //                     msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                         recipient: "depositor".to_string(),
// //                         amount: Uint128::from(100u64),
// //                     })
// //                     .unwrap(),
// //                     funds: vec![]
// //                 })),
// //             ]
// //         );
// //         assert_eq!(
// //             unbond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Unbond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "100"),
// //                 attr("total_bonded", "9999800"),
// //                 attr("claimed_rewards", "40"),
// //             ]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(0u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(100u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_19, state_.last_distributed);
// //         assert_eq!(Uint256::from(9999800u64), state_.total_bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(40000200002u64, 10000000000000000u64),
// //             state_.global_reward_index
// //         );
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(9999800u64), user_position_.bond_amount);
// //         assert_eq!(
// //             Decimal256::from_ratio(40000200002u64, 10000000000000000u64),
// //             user_position_.reward_index
// //         );
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: UN-Staking when reward distribution is live & don't claim rewards (spans multiple blocks) ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_37),
// //             ..Default::default()
// //         });
// //         unbond_msg = ExecuteMsg::Unbond {
// //             amount: Uint256::from(300u64),
// //             withdraw_pending_reward: Some(false),
// //         };
// //         let unbond_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), unbond_msg.clone()).unwrap();
// //         assert_eq!(
// //             unbond_res_s.messages,
// //             vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                 contract_addr: "staking_token".to_string(),
// //                 msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                     recipient: "depositor".to_string(),
// //                     amount: Uint128::from(300u64),
// //                 })
// //                 .unwrap(),
// //                 funds: vec![]
// //             }))]
// //         );
// //         assert_eq!(
// //             unbond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Unbond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "300"),
// //                 attr("total_bonded", "9999500"),
// //                 attr("claimed_rewards", "0"),
// //             ]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(2u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(106u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_37, state_.last_distributed);
// //         assert_eq!(Uint256::from(9999500u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(9999500u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(188u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: UN-Staking when reward distribution is live & claim rewards (spans multiple blocks) ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_39),
// //             ..Default::default()
// //         });
// //         unbond_msg = ExecuteMsg::Unbond {
// //             amount: Uint256::from(100u64),
// //             withdraw_pending_reward: Some(true),
// //         };
// //         let unbond_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), unbond_msg.clone()).unwrap();
// //         assert_eq!(
// //             unbond_res_s.messages,
// //             vec![
// //                 SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                     contract_addr: "whale_token".to_string(),
// //                     msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                         recipient: "depositor".to_string(),
// //                         amount: Uint128::from(209u64),
// //                     })
// //                     .unwrap(),
// //                     funds: vec![]
// //                 })),
// //                 SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                     contract_addr: "staking_token".to_string(),
// //                     msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                         recipient: "depositor".to_string(),
// //                         amount: Uint128::from(100u64),
// //                     })
// //                     .unwrap(),
// //                     funds: vec![]
// //                 })),
// //             ]
// //         );
// //         assert_eq!(
// //             unbond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Unbond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "100"),
// //                 attr("total_bonded", "9999400"),
// //                 attr("claimed_rewards", "209"),
// //             ]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(2u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(106u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_39, state_.last_distributed);
// //         assert_eq!(Uint256::from(9999400u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(9999400u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: UN-Staking when reward distribution is just over & claim rewards ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_001_15),
// //             ..Default::default()
// //         });
// //         unbond_msg = ExecuteMsg::Unbond {
// //             amount: Uint256::from(100u64),
// //             withdraw_pending_reward: Some(true),
// //         };
// //         let unbond_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), unbond_msg.clone()).unwrap();
// //         assert_eq!(
// //             unbond_res_s.messages,
// //             vec![
// //                 SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                     contract_addr: "whale_token".to_string(),
// //                     msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                         recipient: "depositor".to_string(),
// //                         amount: Uint128::from(836u64),
// //                     })
// //                     .unwrap(),
// //                     funds: vec![]
// //                 })),
// //                 SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                     contract_addr: "staking_token".to_string(),
// //                     msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                         recipient: "depositor".to_string(),
// //                         amount: Uint128::from(100u64),
// //                     })
// //                     .unwrap(),
// //                     funds: vec![]
// //                 })),
// //             ]
// //         );
// //         assert_eq!(
// //             unbond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Unbond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "100"),
// //                 attr("total_bonded", "9999300"),
// //                 attr("claimed_rewards", "836"),
// //             ]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(10u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(0u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_001_10, state_.last_distributed);
// //         assert_eq!(Uint256::from(9999300u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(9999300u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test :: UN-Staking when reward distribution is over & claim rewards ***
// //         // ***

// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_001_45),
// //             ..Default::default()
// //         });
// //         unbond_msg = ExecuteMsg::Unbond {
// //             amount: Uint256::from(100u64),
// //             withdraw_pending_reward: Some(true),
// //         };
// //         let unbond_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), unbond_msg.clone()).unwrap();
// //         assert_eq!(
// //             unbond_res_s.messages,
// //             vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                 contract_addr: "staking_token".to_string(),
// //                 msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                     recipient: "depositor".to_string(),
// //                     amount: Uint128::from(100u64),
// //                 })
// //                 .unwrap(),
// //                 funds: vec![]
// //             })),]
// //         );
// //         assert_eq!(
// //             unbond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Unbond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "100"),
// //                 attr("total_bonded", "9999200"),
// //                 attr("claimed_rewards", "0"),
// //             ]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(10u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(0u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_001_10, state_.last_distributed);
// //         assert_eq!(Uint256::from(9999200u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(9999200u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);
// //     }

// //     #[test]
// //     fn test_claim_rewards() {
// //         let mut info = mock_info("staking_token");
// //         let mut deps = th_setup(&[]);

// //         // ***
// //         // *** Test :: Staking before reward distribution goes live ***
// //         // ***

// //         let mut env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_03),
// //             ..Default::default()
// //         });

// //         let amount_to_stake = 1000u128;
// //         let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
// //             msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
// //             sender: "depositor".to_string(),
// //             amount: Uint128::new(amount_to_stake.clone()),
// //         });
// //         let bond_res_s = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
// //         assert_eq!(
// //             bond_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Bond"),
// //                 attr("user", "depositor"),
// //                 attr("amount", "1000"),
// //                 attr("total_bonded", "1000"),
// //             ]
// //         );

// //         // ***
// //         // *** Test #1 :: Claim Rewards  ***
// //         // ***
// //         info = mock_info("depositor");
// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_23),
// //             ..Default::default()
// //         });
// //         let mut claim_msg = ExecuteMsg::Claim {};
// //         let mut claim_res_s =
// //             execute(deps.as_mut(), env.clone(), info.clone(), claim_msg.clone()).unwrap();
// //         assert_eq!(
// //             claim_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Claim"),
// //                 attr("user", "depositor"),
// //                 attr("claimed_rewards", "130"),
// //             ]
// //         );
// //         assert_eq!(
// //             claim_res_s.messages,
// //             vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                 contract_addr: "whale_token".to_string(),
// //                 msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                     recipient: "depositor".to_string(),
// //                     amount: Uint128::from(130u64),
// //                 })
// //                 .unwrap(),
// //                 funds: vec![]
// //             })),]
// //         );
// //         let mut state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(1u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(103u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_23, state_.last_distributed);
// //         assert_eq!(Uint256::from(1000u64), state_.total_bond_amount);
// //         // Check User State
// //         let mut user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(1000u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test #2 :: Claim Rewards  ***
// //         // ***
// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_73),
// //             ..Default::default()
// //         });
// //         claim_msg = ExecuteMsg::Claim {};
// //         claim_res_s = execute(deps.as_mut(), env.clone(), info.clone(), claim_msg.clone()).unwrap();
// //         assert_eq!(
// //             claim_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Claim"),
// //                 attr("user", "depositor"),
// //                 attr("claimed_rewards", "550"),
// //             ]
// //         );
// //         assert_eq!(
// //             claim_res_s.messages,
// //             vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                 contract_addr: "whale_token".to_string(),
// //                 msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                     recipient: "depositor".to_string(),
// //                     amount: Uint128::from(550u64),
// //                 })
// //                 .unwrap(),
// //                 funds: vec![]
// //             })),]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(6u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(118u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_000_73, state_.last_distributed);
// //         assert_eq!(Uint256::from(1000u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(1000u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test #3:: Claim Rewards  ***
// //         // ***
// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_001_13),
// //             ..Default::default()
// //         });
// //         claim_msg = ExecuteMsg::Claim {};
// //         claim_res_s = execute(deps.as_mut(), env.clone(), info.clone(), claim_msg.clone()).unwrap();
// //         assert_eq!(
// //             claim_res_s.attributes,
// //             vec![
// //                 attr("action", "Staking::ExecuteMsg::Claim"),
// //                 attr("user", "depositor"),
// //                 attr("claimed_rewards", "455"),
// //             ]
// //         );
// //         assert_eq!(
// //             claim_res_s.messages,
// //             vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
// //                 contract_addr: "whale_token".to_string(),
// //                 msg: to_binary(&Cw20ExecuteMsg::Transfer {
// //                     recipient: "depositor".to_string(),
// //                     amount: Uint128::from(455u64),
// //                 })
// //                 .unwrap(),
// //                 funds: vec![]
// //             })),]
// //         );
// //         state_ = STATE.load(&deps.storage).unwrap();
// //         assert_eq!(10u64, state_.current_cycle);
// //         assert_eq!(Uint256::from(0u64), state_.current_cycle_rewards);
// //         assert_eq!(1_000_001_10, state_.last_distributed);
// //         assert_eq!(Uint256::from(1000u64), state_.total_bond_amount);
// //         // Check User State
// //         user_position_ = STAKER_INFO
// //             .load(&deps.storage, &Addr::unchecked("depositor"))
// //             .unwrap();
// //         assert_eq!(Uint256::from(1000u64), user_position_.bond_amount);
// //         assert_eq!(Uint256::from(0u64), user_position_.pending_reward);

// //         // ***
// //         // *** Test #4:: Claim Rewards  ***
// //         // ***
// //         env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_001_53),
// //             ..Default::default()
// //         });
// //         claim_msg = ExecuteMsg::Claim {};
// //         let claim_res_f = execute(deps.as_mut(), env.clone(), info.clone(), claim_msg.clone());
// //         assert_generic_error_message(claim_res_f, "No rewards to claim");
// //     }

// //     fn th_setup(contract_balances: &[Coin]) -> OwnedDeps<MockStorage, MockApi, MarsMockQuerier> {
// //         let mut deps = mock_dependencies(contract_balances);
// //         let info = mock_info("owner");
// //         let env = mock_env(MockEnvParams {
// //             block_time: Timestamp::from_seconds(1_000_000_00),
// //             ..Default::default()
// //         });
// //         // Config with valid base params
// //         let instantiate_msg = InstantiateMsg {
// //             owner: Some("owner".to_string()),
// //             whale_token: Some("whale_token".to_string()),
// //             staking_token: Some("staking_token".to_string()),
// //             init_timestamp: 1_000_000_10,
// //             till_timestamp: 1_000_001_10,
// //             cycle_rewards: Some(Uint256::from(100u64)),
// //             cycle_duration: 10u64,
// //             reward_increase: Some(Decimal256::from_ratio(3u64, 100u64)),
// //         };
// //         instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
// //         deps
// //     }
// // }
