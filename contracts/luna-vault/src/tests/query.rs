use cosmwasm_std::from_binary;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::Decimal;

use white_whale::luna_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::luna_vault::msg::*;

use crate::contract::query;
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
pub fn test_config_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    query(deps.as_ref(), env, QueryMsg::PoolConfig {}).unwrap();
}

#[test]
pub fn test_fees_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: FeeResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Fees {}).unwrap()).unwrap();
    assert_eq!(q_res.fees.treasury_fee.share, Decimal::percent(10u64));
    assert_eq!(q_res.fees.flash_loan_fee.share, Decimal::permille(5u64));
}
