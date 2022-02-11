use crate::contract::query;
use crate::tests::tests::mock_instantiate;
use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, Api};
use cw_controllers::AdminResponse;
use white_whale::community_fund::msg::{ConfigResponse, QueryMsg};
use white_whale::treasury::dapp_base::common_test::TEST_CREATOR;

#[test]
fn test_config_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        q_res.token_addr,
        deps.api.addr_validate(MOCK_CONTRACT_ADDR).unwrap()
    )
}

#[test]
fn test_admin_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: AdminResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Admin {}).unwrap()).unwrap();
    assert_eq!(
        q_res.admin.unwrap(),
        deps.api.addr_validate(TEST_CREATOR).unwrap()
    )
}
