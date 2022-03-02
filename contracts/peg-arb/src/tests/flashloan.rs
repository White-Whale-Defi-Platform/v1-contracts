use std::str::FromStr;

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Decimal, Uint128};

use crate::contract::execute;

use terraswap::asset::{Asset, AssetInfo};

use crate::error::StableArbError;
use crate::tests::common::{POOL_NAME, TEST_CREATOR, VAULT_ASSET};
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;
use white_whale::peg_arb::msg::*;

const OFFER_AMOUNT: u64 = 100_000_000u64;

#[test]
fn successfull_flashloan_call() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = mock_info(TEST_CREATOR, &[]);
    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: VAULT_ASSET.to_string(),
            },
        },
        pool_name: POOL_NAME.to_string(),
        slippage: Decimal::percent(1),
        belief_price: Decimal::from_str("420").unwrap(),
    };

    let msg = ExecuteMsg::ExecuteArb {
        details: arb_detail,
        above_peg: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn unsuccessful_flashloan_call_wrong_denom() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update admin
    let info = mock_info(TEST_CREATOR, &[]);
    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: "ukrt".to_string(),
            },
        },
        pool_name: POOL_NAME.to_string(),
        slippage: Decimal::percent(1),
        belief_price: Decimal::from_str("420").unwrap(),
    };

    let msg = ExecuteMsg::ExecuteArb {
        details: arb_detail,
        above_peg: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Ok(_) => panic!("Must return error"),
        // Throws error from deposit_info.rs
        Err(StableArbError::Std(_)) => (),
        Err(_) => panic!("Unknown Error"),
    }
}
