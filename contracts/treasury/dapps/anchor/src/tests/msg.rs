use cosmwasm_std::{Addr};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

use white_whale::memory::item::Memory;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::msg::BaseExecuteMsg;
use white_whale::treasury::dapp_base::state::{ADMIN, BaseState, BASESTATE};
use white_whale_testing::dapp_base::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT, MEMORY_CONTRACT};

use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::tests::base_mocks::mocks::mock_instantiate;


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