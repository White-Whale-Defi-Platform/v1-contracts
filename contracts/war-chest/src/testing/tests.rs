
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Uint128};



#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        gov_contract: "gov".to_string(),
        whale_token: "whale".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap()).unwrap();
    assert_eq!("gov", config.gov_contract.as_str());
    assert_eq!("whale", config.whale_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);
}