use cosmwasm_std::{from_binary, StdResult};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

use white_whale::treasury::dapp_base::msg::{BaseExecuteMsg, BaseQueryMsg, BaseStateResponse};

use crate::contract::{execute, query};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use crate::tests::instantiate::mock_instantiate;

#[test]
pub fn test_config_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: BaseStateResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Base(BaseQueryMsg::Config {})).unwrap())
            .unwrap();

    assert_eq!(
        q_res,
        BaseStateResponse {
            treasury_address: TREASURY_CONTRACT.to_string(),
            trader: TRADER_CONTRACT.to_string(),
        }
    )
}

#[test]
#[should_panic]
pub fn test_address_book_nonexisting_key_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let _q_res: StdResult<String> = from_binary(
        &query(
            deps.as_ref(),
            env,
            QueryMsg::Base(BaseQueryMsg::AddressBook {
                id: "non-existing".to_string(),
            }),
        )
            .unwrap(),
    );
}

#[test]
pub fn test_address_book_existing_key_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateAddressBook {
        to_add: vec![("asset".to_string(), "new_address".to_string())],
        to_remove: vec![],
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let q_res: String = from_binary(
        &query(
            deps.as_ref(),
            env,
            QueryMsg::Base(BaseQueryMsg::AddressBook {
                id: "asset".to_string(),
            }),
        )
            .unwrap(),
    )
        .unwrap();
    assert_eq!(q_res, "new_address".to_string());
}
