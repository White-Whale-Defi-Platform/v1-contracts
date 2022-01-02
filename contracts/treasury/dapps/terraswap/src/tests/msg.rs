
use cosmwasm_std::testing::{mock_env, mock_info};


use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale_testing::dapp_base::common::{TRADER_CONTRACT};

use crate::contract::execute;
use crate::error::TerraswapError;
use crate::msg::ExecuteMsg;
use crate::tests::base_mocks::mocks::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;
use white_whale_testing::dapp_base::common::{WHALE_TOKEN, WHALE_UST_PAIR, WHALE_UST_LP_TOKEN};

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
        Err(TerraswapError::BaseDAppError(BaseDAppError::Unauthorized {})) => (),
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
        Err(TerraswapError::Std(_)) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}

#[test]
pub fn test_successfully_provide_liquidity_existing_asset_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));

    let env = mock_env();
    let msg = ExecuteMsg::ProvideLiquidity {
        pool_id: "pool".to_string(),
        main_asset_id: "asset".to_string(),
        amount: Default::default(),
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
}

#[test]
pub fn test_successfully_provide_detailed_liquidity_existing_asset_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));

    let env = mock_env();
    let msg = ExecuteMsg::DetailedProvideLiquidity {
        pool_id: "pool".to_string(),
        assets: vec![("asset".to_string(), Uint128::from(10u64)), ("asset".to_string(), Uint128::from(10u64))],
        slippage_tolerance: Default::default(),
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
}

#[test]
/// Test to confirm that we can use DetailedProvideLiquidity to provide
/// some assets and then use WithdrawLiqudity to again withdraw those assets. 
/// The balances for WHALE_TOKEN and WHALE_UST_LP_TOKEN are mocked and do not reflect real values
/// Interactions of these dapps can be tested via integration tests
pub fn test_successfully_withdraw_liqudity(){
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("whale_ust".to_string(), WHALE_UST_LP_TOKEN.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("whale_ust_pair".to_string(), WHALE_UST_PAIR.to_string()));


    let env = mock_env();
    let msg = ExecuteMsg::DetailedProvideLiquidity {
        pool_id: "pool".to_string(),
        assets: vec![("asset".to_string(), Uint128::from(10u64)), ("asset".to_string(), Uint128::from(10u64))],
        slippage_tolerance: Default::default(),
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    println!("{:?}", res.events);
    println!("{:?}", res.messages);

    let msg = ExecuteMsg::WithdrawLiquidity{
        lp_token_id: "whale_ust".to_string(),
        amount: Uint128::new(1),
    };
    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(res.messages.len(), 1)
}

#[test]
pub fn test_successful_astro_swap(){
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), WHALE_TOKEN.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("pool".to_string(), WHALE_UST_PAIR.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("whale_ust".to_string(), WHALE_UST_LP_TOKEN.to_string()));
    mock_add_to_address_book(deps.as_mut(), ("whale_ust_pair".to_string(), WHALE_UST_PAIR.to_string()));


    let env = mock_env();
    let msg = ExecuteMsg::SwapAsset {
        pool_id: "pool".to_string(),
        offer_id: "asset".to_string(),
        amount: Uint128::new(1),
        max_spread: None,
        belief_price: None,
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    assert_eq!(res.messages.len(), 1);
}

