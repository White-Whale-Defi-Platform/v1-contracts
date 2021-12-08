use cosmwasm_std::testing::{mock_env, mock_info};

use dapp_template::tests::mocks::{mock_add_to_address_book, mock_instantiate};
use ExecuteMsg::ProvideLiquidity;
use white_whale::treasury::dapp_base::error::DAppError;

use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::tests::mock_querier::mock_dependencies;

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
        Err(DAppError::Unauthorized {}) => (),
        Ok(_) => panic!("Should return unauthorized Error, DAppError::Unauthorized"),
        _ => panic!("Should return unauthorized Error, DAppError::Unauthorized"),
    }
}

#[test]
pub fn test_successfully_provide_liquidity_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_add_to_address_book(deps.as_mut(), ("asset".to_string(), "asset_address".to_string()));

    let env = mock_env();
    let msg = ExecuteMsg::ProvideLiquidity {
        pool_id: "asset".to_string(),
        main_asset_id: "".to_string(),
        amount: Default::default(),
    };

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

}
