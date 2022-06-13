use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Addr, MessageInfo, Response,
};
use cw_controllers::{AdminError, AdminResponse};
use white_whale::luna_vault::luna_unbond_handler::msg::ExecuteMsg;

use crate::{contract::execute, state::ADMIN, tests::common::TEST_CREATOR, UnbondHandlerError};

use super::{common::mock_creator_info, instantiate::mock_instantiate};

#[test]
fn does_set_admin_on_instantiate() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // admin should be sent to the address which instantiated contract
    assert_eq!(
        ADMIN.query_admin(deps.as_ref()).unwrap(),
        AdminResponse {
            admin: Some(TEST_CREATOR.into())
        }
    );
}

#[test]
fn can_update_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    execute(
        deps.as_mut(),
        mock_env(),
        mock_creator_info(),
        ExecuteMsg::SetAdmin {
            admin: "new_owner".into(),
        },
    )
    .unwrap();

    // check that admin changed
    let new_admin = ADMIN.get(deps.as_ref()).unwrap();
    assert_eq!(new_admin, Some(Addr::unchecked("new_owner")));
}

#[test]
fn update_admin_does_return_attributes() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_creator_info(),
        ExecuteMsg::SetAdmin {
            admin: "new_owner".into(),
        },
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_attributes(vec![
            ("previous_admin", TEST_CREATOR),
            ("new_admin", "new_owner")
        ])
    )
}

#[test]
fn only_current_admin_can_reassign() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let malicious_info = MessageInfo {
        sender: Addr::unchecked("malicious_user"),
        funds: vec![],
    };

    let err = execute(
        deps.as_mut(),
        mock_env(),
        malicious_info.clone(),
        ExecuteMsg::SetAdmin {
            admin: malicious_info.sender.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, UnbondHandlerError::Admin(AdminError::NotAdmin {}));
}
