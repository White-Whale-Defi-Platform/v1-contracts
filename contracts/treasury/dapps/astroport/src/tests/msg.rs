use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Addr;
use white_whale::memory::item::Memory;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::msg::BaseExecuteMsg;
use white_whale::treasury::dapp_base::state::{BaseState, ADMIN, BASESTATE};

use crate::contract::execute;
use crate::error::AstroportError;
use crate::msg::ExecuteMsg;
use crate::tests::base_mocks::mocks::mock_instantiate;
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT};
use crate::tests::mock_querier::mock_dependencies;
use white_whale_testing::dapp_base::common::MEMORY_CONTRACT;

/**
 * BaseExecuteMsg::UpdateConfig
 */
#[test]
pub fn test_unsuccessfully_update_config_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
        trader: None,
        memory: None,
    });

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(AstroportError::BaseDAppError(BaseDAppError::Admin(_))) => (),
        Ok(_) => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
        err => panic!(
            "Should return unauthorized Error, Admin(NotAdmin) {:?}",
            err
        ),
    }
}

#[test]
pub fn test_successfully_update_config_treasury_address_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("new_treasury_address".to_string()),
        trader: None,
        memory: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked("new_treasury_address".to_string()),
            trader: Addr::unchecked(TRADER_CONTRACT.to_string()),
            memory: Memory {
                address: Addr::unchecked(&MEMORY_CONTRACT.to_string())
            }
        }
    )
}

// #[test]
// pub fn test_successfully_update_config_none_msg() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());
//     let env = mock_env();
//     let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
//         treasury_address: None,
//         trader: None,
//         memory: None,
//     });

//     let info = mock_info(TEST_CREATOR, &[]);
//     execute(deps.as_mut(), env.clone(), info, msg).unwrap();

//     let state = STATE.load(deps.as_mut().storage).unwrap();

//     assert_eq!(
//         state,
//         BaseState {
//             treasury_address: Addr::unchecked("new_treasury_address".to_string()),
//             trader: Addr::unchecked(TRADER_CONTRACT.to_string()),
//             memory: Memory {
//                 address: Addr::unchecked(&MEMORY_CONTRACT.to_string())
//             }
//         }
//     )
// }

/**
 * BaseExecuteMsg::SetAdmin
 */
#[test]
pub fn test_unsuccessfully_set_admin_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::SetAdmin {
        admin: "new_admin".to_string(),
    });

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(AstroportError::BaseDAppError(BaseDAppError::Admin(_))) => (),
        Ok(_) => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
        _ => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
    }
}

#[test]
pub fn test_successfully_set_admin_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    // check original admin
    let admin = ADMIN.get(deps.as_ref()).unwrap().unwrap();
    assert_eq!(admin, Addr::unchecked(TEST_CREATOR.to_string()));

    // set new admin
    let msg = ExecuteMsg::Base(BaseExecuteMsg::SetAdmin {
        admin: "new_admin".to_string(),
    });
    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check new admin
    let admin = ADMIN.get(deps.as_ref()).unwrap().unwrap();
    assert_eq!(admin, Addr::unchecked("new_admin".to_string()));
}

/**
 * ExecuteMsg::ProvideLiquidity
 */
#[test]
pub fn test_provide_liquidity_unauthorized_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::ProvideLiquidity {
        pool_id: "".to_string(),
        main_asset_id: "".to_string(),
        amount: Default::default(),
    };

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(AstroportError::BaseDAppError(BaseDAppError::Unauthorized {})) => (),
        Ok(_) => panic!("Should return unauthorized Error, DAppError::Unauthorized"),
        _ => panic!("Should return unauthorized Error, DAppError::Unauthorized"),
    }
}

#[test]
pub fn test_successfully_provide_liquidity_nonexisting_asset_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let env = mock_env();
    let msg = ExecuteMsg::ProvideLiquidity {
        pool_id: "asset".to_string(),
        main_asset_id: "".to_string(),
        amount: Default::default(),
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(AstroportError::Std(_)) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}

// #[test]
// pub fn test_successfully_provide_liquidity_existing_asset_msg() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());
//     mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));

//     let env = mock_env();
//     let msg = ExecuteMsg::ProvideLiquidity {
//         pool_id: "pool".to_string(),
//         main_asset_id: "asset".to_string(),
//         amount: Default::default(),
//     };

//     let info = mock_info(TRADER_CONTRACT, &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
// }

// #[test]
// pub fn test_successfully_provide_detailed_liquidity_existing_asset_msg() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());
//     mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));

//     let env = mock_env();
//     let msg = ExecuteMsg::DetailedProvideLiquidity {
//         pool_id: "pool".to_string(),
//         assets: vec![("asset".to_string(), Uint128::from(10u64)), ("asset".to_string(), Uint128::from(10u64))],
//         slippage_tolerance: Default::default(),
//     };

//     let info = mock_info(TRADER_CONTRACT, &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
// }

// #[test]
// Test to confirm that we can use DetailedProvideLiquidity to provide
// some assets and then use WithdrawLiqudity to again withdraw those assets.
// The balances for WHALE_TOKEN and WHALE_UST_LP_TOKEN are mocked and do not reflect real values
// Interactions of these dapps can be tested via integration tests
// pub fn test_successfully_withdraw_liqudity(){
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());
//     mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("whale_ust".to_string(), WHALE_UST_LP_TOKEN.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("whale_ust_pair".to_string(), WHALE_UST_PAIR.to_string()));

//     let env = mock_env();
//     let msg = ExecuteMsg::DetailedProvideLiquidity {
//         pool_id: "pool".to_string(),
//         assets: vec![("asset".to_string(), Uint128::from(10u64)), ("asset".to_string(), Uint128::from(10u64))],
//         slippage_tolerance: Default::default(),
//     };

//     let info = mock_info(TRADER_CONTRACT, &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
//     println!("{:?}", res.events);
//     println!("{:?}", res.messages);

//     let msg = ExecuteMsg::WithdrawLiquidity{
//         lp_token_id: "whale_ust".to_string(),
//         amount: Uint128::new(1),
//     };
//     let info = mock_info(TRADER_CONTRACT, &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

//     assert_eq!(res.messages.len(), 1)
// }

// #[test]
// pub fn test_successful_astro_swap(){
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());
//     mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("whale_ust".to_string(), WHALE_UST_LP_TOKEN.to_string()));
//     mock_add_to_address_book(deps.as_mut(), ("whale_ust_pair".to_string(), WHALE_UST_PAIR.to_string()));

//     let env = mock_env();
//     let msg = ExecuteMsg::SwapAsset {
//         pool_id: "pool".to_string(),
//         offer_id: "asset".to_string(),
//         amount: Uint128::new(1),
//         max_spread: None,
//         belief_price: None,
//     };

//     let info = mock_info(TRADER_CONTRACT, &[]);
//     let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

//     assert_eq!(res.messages.len(), 1);
// }
