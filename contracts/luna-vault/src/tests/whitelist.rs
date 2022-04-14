use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Api, MessageInfo};

use white_whale::memory::LIST_SIZE_LIMIT;
use white_whale::ust_vault::msg::ExecuteMsg;

use crate::contract::execute;
use crate::error::StableVaultError;
use crate::state::{State, STATE};
use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR};
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

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
fn unsuccessful_add_to_whitelist_limit_exceeded() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let info = mock_info(TEST_CREATOR, &[]);

    for n in 0..LIST_SIZE_LIMIT + 1 {
        let mut contract_addr = "contract".to_owned();
        let number = n.to_string().to_owned();
        contract_addr.push_str(&number);

        let msg = ExecuteMsg::AddToWhitelist { contract_addr };

        match execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()) {
            Ok(_) => {
                let state: State = STATE.load(&deps.storage).unwrap();
                assert!(state.whitelisted_contracts.len() <= LIST_SIZE_LIMIT);
            }
            Err(StableVaultError::WhitelistLimitReached {}) => {
                let state: State = STATE.load(&deps.storage).unwrap();
                assert_eq!(state.whitelisted_contracts.len(), LIST_SIZE_LIMIT);
                ()
            } //expected at n > LIST_SIZE_LIMIT
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
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
        deps.api.addr_validate(ARB_CONTRACT).unwrap(),
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
        deps.api.addr_validate(ARB_CONTRACT).unwrap(),
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
