use cosmwasm_std::{Addr, StdError};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::msg::BaseExecuteMsg;
use white_whale::treasury::dapp_base::state::{ADMIN, BaseState, load_contract_addr, STATE};
use white_whale::treasury::dapp_base::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};

use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::tests::mocks::mock_instantiate;

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
    });

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::Admin(_)) => (),
        Ok(_) => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
        _ => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
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
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = STATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked("new_treasury_address".to_string()),
            trader: Addr::unchecked(TRADER_CONTRACT.to_string()),
        }
    )
}

#[test]
pub fn test_successfully_update_config_trader_address_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
        trader: Some("new_trader_address".to_string()),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = STATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked(TREASURY_CONTRACT.to_string()),
            trader: Addr::unchecked("new_trader_address".to_string()),
        }
    )
}

#[test]
pub fn test_successfully_update_config_both_treasury_and_trader_address_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("new_treasury_address".to_string()),
        trader: Some("new_trader_address".to_string()),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = STATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked("new_treasury_address".to_string()),
            trader: Addr::unchecked("new_trader_address".to_string()),
        }
    )
}

#[test]
pub fn test_successfully_update_config_none_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
        trader: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = STATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked(TREASURY_CONTRACT.to_string()),
            trader: Addr::unchecked(TRADER_CONTRACT.to_string()),
        }
    )
}

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
        Err(BaseDAppError::Admin(_)) => (),
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
 * BaseExecuteMsg::UpdateAddressBook
 */
#[test]
pub fn test_unsuccessfully_update_address_book_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
        to_add: vec![],
        to_remove: vec![],
    });

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::Admin(_)) => (),
        Ok(_) => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
        _ => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
    }
}

#[test]
pub fn test_successfully_update_address_book_add_address_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
        to_add: vec![("asset".to_string(), "address".to_string())],
        to_remove: vec![],
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let asset_address = load_contract_addr(deps.as_ref(), "asset").unwrap();
    assert_eq!(asset_address, Addr::unchecked("address".to_string()));
}

#[test]
pub fn test_successfully_update_address_book_remove_address_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    // add address
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
        to_add: vec![("asset".to_string(), "address".to_string())],
        to_remove: vec![],
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let asset_address = load_contract_addr(deps.as_ref(), "asset").unwrap();
    assert_eq!(asset_address, Addr::unchecked("address".to_string()));

    // remove address
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
        to_add: vec![],
        to_remove: vec!["asset".to_string()],
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = load_contract_addr(deps.as_ref(), "asset");

    match res {
        Err(StdError::NotFound { .. }) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}


#[test]
pub fn test_successfully_update_address_book_add_and_removeaddress_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    //add address
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
        to_add: vec![("asset".to_string(), "address".to_string())],
        to_remove: vec![],
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let asset_address = load_contract_addr(deps.as_ref(), "asset").unwrap();
    assert_eq!(asset_address, Addr::unchecked("address".to_string()));

    // query non-existing address
    let res = load_contract_addr(deps.as_ref(), "another_asset");
    match res {
        Err(StdError::NotFound { .. }) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }

    //add and remove addresses
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
        to_add: vec![("another_asset".to_string(), "another_address".to_string())],
        to_remove: vec!["asset".to_string()],
    });
    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // another_asset should be in the addressbook now
    let asset_address = load_contract_addr(deps.as_ref(), "another_asset").unwrap();
    assert_eq!(asset_address, Addr::unchecked("another_address".to_string()));

    // asset should not be in the addressbook now
    let res = load_contract_addr(deps.as_ref(), "asset");
    match res {
        Err(StdError::NotFound { .. }) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}
