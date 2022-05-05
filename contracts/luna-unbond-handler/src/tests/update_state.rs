use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Addr,
};
use white_whale::luna_vault::luna_unbond_handler::msg::ExecuteMsg;

use crate::{
    contract::execute,
    state::{State, STATE},
};

use super::{
    common::{mock_creator_info, TEST_MEMORY_CONTRACT},
    instantiate::mock_instantiate,
};

#[test]
fn does_update_owner() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    execute(
        deps.as_mut(),
        mock_env(),
        mock_creator_info(),
        ExecuteMsg::UpdateState {
            // modify owner
            owner: Some("new_owner".into()),
            // keep remaining fields the same
            expiration_time: None,
            memory_contract: None,
        },
    )
    .unwrap();

    let state = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            owner: Some(Addr::unchecked("new_owner")),
            expiration_time: Some(mock_env().block.time.seconds()),
            memory_contract: Addr::unchecked(TEST_MEMORY_CONTRACT),
        }
    )
}
