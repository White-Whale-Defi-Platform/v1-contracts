use std::panic;

use crate::contract::{execute, instantiate};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Uint128, Addr, QuerierWrapper};
use terraswap::asset::{AssetInfo, Asset};
use white_whale::treasury::msg::{ExecuteMsg, InstantiateMsg};
use crate::tests::common::TEST_CREATOR;
use crate::error::*;

const NOT_ALLOWED: &str = "some_other_contract";

fn init_msg() -> InstantiateMsg {
    InstantiateMsg {}
}

#[test]
fn test_non_whitelisted() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddTrader {
        trader: TEST_CREATOR.to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }

    let test_token = Asset {
            info: AssetInfo::Token{
                contract_addr: "test_token".to_string()
            },
            amount: Uint128::zero()
        };
        
    let info = mock_info(NOT_ALLOWED, &[]);

    let msg = ExecuteMsg::TraderAction {
        msgs: vec![test_token.into_msg(&QuerierWrapper::new(&deps.querier),Addr::unchecked(NOT_ALLOWED)).unwrap()]
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => panic!("Sender should not be allowed to do this action"),
        Err(e) => match e {
            TreasuryError::SenderNotWhitelisted {} => (),
            _ => panic!("Unknown error: {}", e),
        },
    }
}
