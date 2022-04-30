use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, DepsMut, Response,
};
use cw2::{ContractVersion, CONTRACT};

use crate::{
    contract::{instantiate, CONTRACT_NAME, CONTRACT_VERSION},
    msg::InstantiateMsg,
    state::{State, STATE},
};

use super::common::{mock_creator_info, TEST_CREATOR, TEST_MEMORY_CONTRACT, TEST_OWNER};

/// Mocks instantiation of the contract
pub fn mock_instantiate(deps: DepsMut) -> Response {
    let msg = InstantiateMsg {
        owner: Some(TEST_OWNER.into()),
        memory_contract: TEST_MEMORY_CONTRACT.into(),
    };

    instantiate(deps, mock_env(), mock_creator_info(), msg).expect("Contract failed init")
}

#[test]
fn initialization_does_save_state() {
    let mut deps = mock_dependencies(&[]);

    mock_instantiate(deps.as_mut());

    let expiration_time = mock_env().block.time.seconds();

    let state = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            owner: Some(Addr::unchecked(TEST_OWNER)),
            memory_contract: Addr::unchecked(TEST_MEMORY_CONTRACT),
            expiration_time: Some(expiration_time),
        }
    )
}

#[test]
fn initialization_does_return_attributes() {
    let mut deps = mock_dependencies(&[]);

    let res = mock_instantiate(deps.as_mut());

    let expiration_time = mock_env().block.time.seconds();

    assert_eq!(
        res,
        Response::new().add_attributes(vec![
            ("method", "instantiate"),
            ("owner", TEST_OWNER),
            ("expiration_time", &expiration_time.to_string())
        ])
    )
}

#[test]
fn initialization_does_set_version() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let version = CONTRACT.load(&deps.storage).unwrap();
    assert_eq!(
        version,
        ContractVersion {
            contract: CONTRACT_NAME.into(),
            version: CONTRACT_VERSION.into(),
        }
    )
}
