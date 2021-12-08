use cosmwasm_std::{StdError, Uint128};
use cosmwasm_std::testing::{mock_env, mock_info};

use dapp_template::tests::mocks::{mock_add_to_address_book, mock_instantiate};
use ExecuteMsg::ProvideLiquidity;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::tests::common::{TEST_CREATOR, TRADER_CONTRACT};

use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::tests::mock_querier::mock_dependencies;
use crate::error::TerraswapError;

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
        Err(TerraswapError::BaseDAppError(BaseDAppError::Std(_))) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}

#[test]
pub fn test_successfully_provide_liquidity_existing_asset_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), "asset_address".to_string()));

    let env = mock_env();
    let msg = ExecuteMsg::ProvideLiquidity {
        pool_id: "asset".to_string(),
        main_asset_id: "".to_string(),
        amount: Default::default(),
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
}

#[test]
pub fn test_successfully_provide_detailed_liquidity_existing_asset_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), "asset_address".to_string()));

    let env = mock_env();
    let msg = ExecuteMsg::DetailedProvideLiquidity {
        pool_id: "pool".to_string(),
        assets: vec![("asset".to_string(),Uint128::from(10u64) ), ("asset".to_string(),Uint128::from(10u64) )],
        slippage_tolerance: Default::default(),
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
}