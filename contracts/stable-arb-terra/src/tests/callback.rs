use std::str::FromStr;

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Decimal, Uint128};

use crate::contract::execute;

use terraswap::asset::{Asset, AssetInfo};

use crate::error::StableArbError;
use crate::msg::*;
use crate::tests::common::{TEST_CREATOR, VAULT_ASSET};
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

const OFFER_AMOUNT: u64 = 100_000_000u64;

#[test]
fn unsuccessfull_self_callback() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = mock_info(TEST_CREATOR, &[]);

    let msg = ExecuteMsg::Callback(CallbackMsg::AfterSuccessfulTradeCallback {});

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Ok(_) => panic!("Must return error"),
        // Caller is not contract itself
        Err(StableArbError::NotCallback {}) => (),
        Err(_) => panic!("Unknown Error"),
    }
}

#[test]
fn unsuccessfull_vault_callback() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = mock_info("some_other_contract", &[]);

    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: VAULT_ASSET.to_string(),
            },
        },
        slippage: Decimal::percent(1),
        belief_price: Decimal::from_str("420").unwrap(),
    };

    let msg = ExecuteMsg::AbovePegCallback {
        details: arb_detail,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Ok(_) => panic!("Must return error"),
        // Caller is not the vault
        Err(StableArbError::Unauthorized {}) => (),
        Err(_) => panic!("Unknown Error"),
    }
}
