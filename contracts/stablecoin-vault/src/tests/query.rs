use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, from_binary};
use cosmwasm_std::{Decimal, Uint128};

use white_whale::ust_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::ust_vault::msg::*;

use crate::contract::query;
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
pub fn test_config_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    // Errors because canonical -> human address conversion errors on fake contract names
    query(deps.as_ref(), env, QueryMsg::PoolConfig {}).unwrap_err();
}

#[test]
pub fn test_state_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: StateResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::State {}).unwrap()).unwrap();
    assert_eq!(q_res.allow_non_whitelisted, false)
}

#[test]
pub fn test_fees_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: FeeResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Fees {}).unwrap()).unwrap();
    assert_eq!(q_res.fees.warchest_fee.share, Decimal::percent(10u64));
    assert_eq!(q_res.fees.flash_loan_fee.share, Decimal::permille(5u64));
}

#[test]
pub fn test_vault_value_query() {
    let mut deps = mock_dependencies(&[coin(1000, "uusd")]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: ValueResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::VaultValue {}).unwrap()).unwrap();
    assert_eq!(q_res.total_ust_value, Uint128::new(1000))
}

#[test]
pub fn test_vault_estimate_fee_query() {
    let mut deps = mock_dependencies(&[coin(1000, "uusd")]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: EstimateWithdrawFeeResponse = from_binary(
        &query(
            deps.as_ref(),
            env,
            QueryMsg::EstimateWithdrawFee {
                amount: Uint128::new(1000),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_ne!(q_res.fee, vec![])
}
