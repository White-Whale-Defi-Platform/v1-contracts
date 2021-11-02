use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Api, MessageInfo};
use cw_controllers::AdminResponse;
use white_whale::community_fund::msg::{ConfigResponse, QueryMsg};
use white_whale::community_fund::msg::QueryMsg::Config;
use crate::contract::{execute, instantiate, query};
use crate::msg::InstantiateMsg;
use crate::state::{STATE, State, state_read};

use super::*;

pub(crate) fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        whale_token_addr: "whale token".to_string(),
    }
}

#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_validate("creator").unwrap(),
        funds: vec![],
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    /*let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        Config {}
    );*/

    let state: State = state_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            whale_token_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
        }
    );
}

#[test]
fn test_config_query() {
    let mut deps = mock_dependencies(&[]);
    let msg = instantiate_msg();
    let env = mock_env();
    let creator_info = MessageInfo {
        sender: deps.api.addr_validate("creator").unwrap(),
        funds: vec![],
    };

    let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    let q_res: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        q_res.token_addr,
        deps.api.addr_validate("whale token").unwrap()
    )
}

#[test]
fn test_admin_query() {
    let mut deps = mock_dependencies(&[]);
    let msg = instantiate_msg();
    let env = mock_env();
    let creator_info = MessageInfo {
        sender: deps.api.addr_validate("creator").unwrap(),
        funds: vec![],
    };

    let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    let q_res: AdminResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Admin {}).unwrap()).unwrap();
    assert_eq!(
        q_res.admin.unwrap(),
        deps.api.addr_validate("creator").unwrap()
    )
}

#[test]
fn test_burn_tokens() {}
