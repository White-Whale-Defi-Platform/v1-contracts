use crate::contract::execute;
use crate::error::StableVaultError;
use crate::state::STATE;
use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR};
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::{mock_dependencies};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Api, MessageInfo};
use white_whale::ust_vault::msg::ExecuteMsg;

/**
 * Tests adding to whitelist
 */
#[test]
fn unsuccessful_add_to_whitelist_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StableVaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}

#[test]
fn unsuccessful_add_to_whitelist_already_whitelisted() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: "contract".to_string(),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // repeat the same whitelisting
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StableVaultError::AlreadyWhitelisted {}) => (),
        _ => panic!("Must return StableVaultError::AlreadyWhitelisted"),
    }
}

#[test]
fn successful_add_to_whitelist() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let whitelisted_contracts = STATE
        .load(deps.as_mut().storage)
        .unwrap()
        .whitelisted_contracts;
    assert_eq!(0, whitelisted_contracts.len());

    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    let whitelisted_contracts = STATE
        .load(deps.as_mut().storage)
        .unwrap()
        .whitelisted_contracts;
    assert_eq!(1, whitelisted_contracts.len());
    assert_eq!(
        deps.api.addr_canonicalize(ARB_CONTRACT).unwrap(),
        whitelisted_contracts[0]
    );
}

/**
 * Tests removing to whitelist
 */
#[test]
fn unsuccessful_remove_from_whitelist_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::RemoveFromWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StableVaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}

#[test]
fn unsuccessful_remove_from_whitelist_not_whitelisted() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::RemoveFromWhitelist {
        contract_addr: "contract".to_string(),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StableVaultError::NotWhitelisted {}) => (),
        _ => panic!("Must return StableVaultError::NotWhitelisted"),
    }
}

#[test]
fn successful_remove_from_whitelist() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    //Add contract to whitelist first
    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let whitelisted_contracts = STATE
        .load(deps.as_mut().storage)
        .unwrap()
        .whitelisted_contracts;
    //one contract should be whitelisted
    assert_eq!(1, whitelisted_contracts.len());
    assert_eq!(
        deps.api.addr_canonicalize(ARB_CONTRACT).unwrap(),
        whitelisted_contracts[0]
    );

    //Remove contract from whitelist
    let msg = ExecuteMsg::RemoveFromWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    let whitelisted_contracts = STATE
        .load(deps.as_mut().storage)
        .unwrap()
        .whitelisted_contracts;
    //no contract should be whitelisted
    assert_eq!(0, whitelisted_contracts.len());
}
