use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coins, from_binary, DepsMut};
use cosmwasm_std::{Api, CanonicalAddr, Decimal, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::state::{State, ADMIN, DEPOSIT_INFO, FEE, POOL_INFO, STATE};
use cw_controllers::Admin;
use cw_storage_plus::Item;
use terraswap::asset::AssetInfo;
use white_whale::fee::*;
use white_whale::ust_vault::msg::*;
use white_whale::ust_vault::msg::VaultQueryMsg as QueryMsg;

use crate::tests::common::{ARB_CONTRACT, LP_TOKEN, TEST_CREATOR};

use crate::error::StableVaultError;
use crate::tests::mock_querier::mock_dependencies;

pub(crate) fn instantiate_msg() -> InitMsg {
    InitMsg {
        anchor_money_market_address: "test_mm".to_string(),
        aust_address: "test_aust".to_string(),
        profit_check_address: "test_profit_check".to_string(),
        community_fund_addr: "community_fund".to_string(),
        warchest_addr: "warchest".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: 0u64,
        warchest_fee: Decimal::percent(10u64),
        community_fund_fee: Decimal::permille(5u64),
        max_community_fund_fee: Uint128::from(1000000u64),
        stable_cap: Uint128::from(100_000_000u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    }
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let msg = InitMsg {
        anchor_money_market_address: "test_mm".to_string(),
        aust_address: "test_aust".to_string(),
        profit_check_address: "test_profit_check".to_string(),
        community_fund_addr: "community_fund".to_string(),
        warchest_addr: "warchest".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: 0u64,
        warchest_fee: Decimal::percent(10u64),
        community_fund_fee: Decimal::permille(5u64),
        max_community_fund_fee: Uint128::from(1000000u64),
        stable_cap: Uint128::from(100_000_000u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res =
        instantiate(deps, mock_env(), info, msg).expect("contract successfully handles InitMsg");
}

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            anchor_money_market_address: deps.api.addr_canonicalize("test_mm").unwrap(),
            aust_address: deps.api.addr_canonicalize("test_aust").unwrap(),
            profit_check_address: deps.api.addr_canonicalize("test_profit_check").unwrap(),
            whitelisted_contracts: vec![],
        }
    );

    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state.whitelisted_contracts[0],
        deps.api.addr_canonicalize(&ARB_CONTRACT).unwrap(),
    );

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            anchor_money_market_address: CanonicalAddr::from(vec![]),
            aust_address: CanonicalAddr::from(vec![]),
            profit_check_address: CanonicalAddr::from(vec![]),
            whitelisted_contracts: vec![deps.api.addr_canonicalize(&ARB_CONTRACT).unwrap()],
        }
    );
}

/**
 * Tests updating the fees of the contract.
 */
#[test]
fn successful_update_fee() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update fees
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::SetFee {
        community_fund_fee: Some(CappedFee {
            fee: Fee {
                share: Decimal::percent(1),
            },
            max_fee: Uint128::from(1_000_000u64),
        }),
        warchest_fee: Some(Fee {
            share: Decimal::percent(2),
        }),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the fee
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Fees {}).unwrap();
    let fee_response: FeeResponse = from_binary(&res).unwrap();
    let fees: VaultFee = fee_response.fees;
    assert_eq!(Decimal::percent(1), fees.community_fund_fee.fee.share);
    assert_eq!(Uint128::from(1_000_000u64), fees.community_fund_fee.max_fee);
    assert_eq!(Decimal::percent(2), fees.warchest_fee.share);

#[test]
fn successfull_set_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update admin
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::SetAdmin {
        admin: "new_admin".to_string();
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());


}