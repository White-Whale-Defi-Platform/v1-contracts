use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Api, DepsMut, MessageInfo};

use white_whale::community_fund::msg::ExecuteMsg;

use crate::contract::{execute, instantiate};
use crate::error::CommunityFundError;
use white_whale::community_fund::msg::InstantiateMsg;
use crate::state::{State, ADMIN, STATE};

const TEST_CREATOR: &str = "creator";

pub(crate) fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        whale_token_addr: MOCK_CONTRACT_ADDR.to_string(),
    }
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let msg = InstantiateMsg {
        whale_token_addr: MOCK_CONTRACT_ADDR.to_string(),
    };

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };
    let _res = instantiate(deps, mock_env(), info.clone(), msg)
        .expect("contract successfully handles InstantiateMsg");
}

#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let state: State = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        State {
            whale_token_addr: deps.api.addr_validate(MOCK_CONTRACT_ADDR).unwrap(),
        }
    );
}

#[test]
fn unsuccessful_set_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let msg = ExecuteMsg::SetAdmin {
        admin: "unauthorized".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(CommunityFundError::Admin(_)) => (),
        _ => panic!("Must return CommunityFundError::Admin"),
    }
}

#[test]
fn successful_set_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let msg = ExecuteMsg::SetAdmin {
        admin: "new admin".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let new_admin = ADMIN.get(deps.as_ref()).unwrap().unwrap();
    assert_eq!(new_admin.as_str(), "new admin");
}
