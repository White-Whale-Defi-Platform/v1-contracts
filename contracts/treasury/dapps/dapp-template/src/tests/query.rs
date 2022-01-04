use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, StdResult};

use white_whale::treasury::dapp_base::msg::{BaseExecuteMsg, BaseQueryMsg, BaseStateResponse};
use white_whale_testing::dapp_base::common::{
    MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT,
};

use crate::contract::{execute, query};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::tests::base_mocks::mocks::mock_instantiate;

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
            memory_address: MEMORY_CONTRACT.to_string(),
        }
    )
}
