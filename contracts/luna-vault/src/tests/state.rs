use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Api, MessageInfo};

use white_whale::luna_vault::msg::ExecuteMsg;

use crate::contract::execute;
use crate::error::LunaVaultError;
use crate::state::STATE;
use crate::tests::common::TEST_CREATOR;

use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

fn state_msg() -> ExecuteMsg {
    ExecuteMsg::UpdateState {
        bluna_address: None,
        cluna_address: None,
        astro_lp_address: None,
        memory_address: None,
        whitelisted_contracts: None,
        allow_non_whitelisted: None,
    }
}

#[test]
fn unsuccessful_set_state_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = state_msg();
    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}

#[test]
fn successful_set_state() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let original_state = STATE.load(deps.as_mut().storage).unwrap();

    let msg = ExecuteMsg::UpdateState {
        bluna_address: Some("newbluna".to_string()),
        cluna_address: None,
        astro_lp_address: None,
        memory_address: None,
        whitelisted_contracts: None,
        allow_non_whitelisted: Some(true),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let new_state = STATE.load(deps.as_mut().storage).unwrap();
    assert_ne!(original_state, new_state);
    assert_eq!(
        new_state.bluna_address,
        deps.api.addr_validate("newbluna").unwrap()
    );
    assert!(new_state.allow_non_whitelisted);
}
