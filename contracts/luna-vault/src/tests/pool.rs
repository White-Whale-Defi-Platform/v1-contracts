use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Api, MessageInfo, Uint128};

use white_whale::luna_vault::msg::ExecuteMsg;

use crate::contract::execute;
use crate::error::LunaVaultError;
use crate::state::POOL_INFO;
use crate::tests::common::TEST_CREATOR;
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
fn unsuccessful_set_stable_cap_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::SetLunaCap {
        luna_cap: Uint128::from(100u128),
    };
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
fn successful_set_stable_cap() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let pool_info = POOL_INFO.load(deps.as_mut().storage).unwrap();
    let original_cap = pool_info.luna_cap;

    let msg = ExecuteMsg::SetLunaCap {
        luna_cap: Uint128::from(100u128),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let pool_info = POOL_INFO.load(deps.as_mut().storage).unwrap();
    assert_eq!(pool_info.luna_cap, Uint128::from(100u128));
    assert_ne!(pool_info.luna_cap, original_cap);
}
