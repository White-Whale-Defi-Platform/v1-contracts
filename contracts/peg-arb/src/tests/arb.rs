use crate::tests::common::{POOL_NAME, TEST_CREATOR, VAULT_CONTRACT};
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, CosmosMsg, Decimal, Uint128};
use terra_cosmwasm::TerraRoute;
use terraswap::asset::{Asset, AssetInfo};
use white_whale::peg_arb::msg::*;

use crate::contract::{execute, instantiate};
use crate::error::StableArbError;

const OFFER_AMOUNT: u64 = 1000u64;

#[test]
fn when_given_a_below_peg_msg_then_handle_returns_first_a_mint_then_a_terraswap_msg() {
    let mut deps = mock_dependencies(&coins(100000000, "uusd"));
    mock_instantiate(deps.as_mut());

    let env = mock_env();

    // Prepare a Mock Arb Detail object
    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
        },
        pool_name: POOL_NAME.to_string(),
        slippage: Decimal::percent(1),
        belief_price: Decimal::percent(420),
    };

    // Prepare a BelowPegCallback msg
    let msg = ExecuteMsg::BelowPegCallback {
        details: arb_detail,
    };

    // Ensure the 'caller' is the VAULT_CONTRACT to avoid unauthorized issues
    let info = mock_info(VAULT_CONTRACT, &[]);

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    // We should have gotten 3 messages back in this case
    assert_eq!(3, res.messages.len());
    // Verify the operations happened in the order we expect.
    // For below peg, we expect first a mint tx, followed by a swap
    let first_msg = res.messages[0].msg.clone();
    match first_msg {
        CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
        CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
        CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected"),
        _ => panic!("unexpected"),
    }
    let second_msg = res.messages[2].msg.clone();
    match second_msg {
        CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
        CosmosMsg::Custom(_t) => panic!("unexpected"),
        CosmosMsg::Wasm(_wasm_msg) => {}
        _ => panic!("unexpected"),
    }
}

#[test]
fn when_given_an_above_peg_msg_then_handle_returns_first_a_terraswap_then_a_mint_msg() {
    let mut deps = mock_dependencies(&coins(100000000, "uusd"));
    mock_instantiate(deps.as_mut());

    let env = mock_env();

    // Prepare a Mock Arb Detail object
    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
        },
        pool_name: POOL_NAME.to_string(),
        slippage: Decimal::percent(1),
        belief_price: Decimal::percent(420),
    };

    // Prepare an AbovePegCallback msg
    let msg = ExecuteMsg::AbovePegCallback {
        details: arb_detail,
    };

    // Ensure the 'caller' is the VAULT_CONTRACT to avoid unauthorized issues
    let info = mock_info(VAULT_CONTRACT, &[]);

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    // We should have gotten 3 messages back in this case
    assert_eq!(3, res.messages.len());
    // Verify the operations happened in the order we expect.
    // For above peg, we expect first terraswap swap tx, followed by a mint
    let first_msg = res.messages[0].msg.clone();
    match first_msg {
        CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
        CosmosMsg::Custom(_t) => panic!("unexpected"),
        CosmosMsg::Wasm(_wasm_msg) => {}
        _ => panic!("unexpected"),
    }
    // Verify the second message is indeed a Market call (to Treasury or otherwise)
    let second_msg = res.messages[1].msg.clone();
    match second_msg {
        CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
        CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
        CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected"),
        _ => panic!("unexpected"),
    }
}


#[test]
fn when_given_a_wrong_arb_complains() {
    let mut deps = mock_dependencies(&coins(100000000, "uusd"));
    mock_instantiate(deps.as_mut());

    let env = mock_env();

    // Prepare a Mock Arb Detail object
    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: "ukrt".to_string(),
            },
        },
        pool_name: POOL_NAME.to_string(),
        slippage: Decimal::percent(1),
        belief_price: Decimal::percent(420),
    };

    // Prepare an AbovePegCallback msg
    let msg = ExecuteMsg::ExecuteArb {
        details: arb_detail,
        above_peg: true
    };

    // Ensure the 'caller' is the VAULT_CONTRACT to avoid unauthorized issues
    let info = mock_info(VAULT_CONTRACT, &[]);

    let res = execute(deps.as_mut(), env, info, msg);
    match res {
        Err(StableArbError::Std(_)) => (),
        _ => panic!("Must return LunaVaultError::Std from DepositInfo::assert"),
    }
}

#[test]
fn peg_arb_can_support_luna_arb_with_differing_msgs() {
    let mut deps = mock_dependencies(&coins(100000000, "uluna"));

    let msg = InstantiateMsg {
        vault_address: VAULT_CONTRACT.to_string(),
        seignorage_address: "seignorage".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uluna".to_string(),
        },
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
        .expect("contract successfully handles InstantiateMsg");

    let add_pool_msg = ExecuteMsg::UpdatePools {
        to_add: Some(vec![(POOL_NAME.to_string(), "terraswap_pool".to_string())]),
        to_remove: None,
    };

    let _res = execute(deps.as_mut(), mock_env(), info, add_pool_msg).unwrap();

    let env = mock_env();

    // Prepare a Mock Arb Detail object
    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
        },
        pool_name: POOL_NAME.to_string(),
        slippage: Decimal::percent(1),
        belief_price: Decimal::percent(420),
    };

    // Prepare an AbovePegCallback msg
    let msg = ExecuteMsg::ExecuteArb {
        details: arb_detail,
        above_peg: true
    };

    // Ensure the 'caller' is the VAULT_CONTRACT to avoid unauthorized issues
    let info = mock_info(VAULT_CONTRACT, &[]);

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    // We should have gotten 1 messages back in this case
    assert_eq!(1, res.messages.len());

}


#[test]
fn peg_arb_can_support_below_or_above_peg_with_luna() {
    let mut deps = mock_dependencies(&coins(100000000, "uluna"));

    let msg = InstantiateMsg {
        vault_address: VAULT_CONTRACT.to_string(),
        seignorage_address: "seignorage".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uluna".to_string(),
        },
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
        .expect("contract successfully handles InstantiateMsg");

    let add_pool_msg = ExecuteMsg::UpdatePools {
        to_add: Some(vec![(POOL_NAME.to_string(), "terraswap_pool".to_string())]),
        to_remove: None,
    };

    let _res = execute(deps.as_mut(), mock_env(), info, add_pool_msg).unwrap();

    let env = mock_env();

    // Prepare a Mock Arb Detail object
    let arb_detail: ArbDetails = ArbDetails {
        asset: Asset {
            amount: Uint128::from(OFFER_AMOUNT),
            info: AssetInfo::NativeToken {
                denom: "uluna".to_string(),
            },
        },
        pool_name: POOL_NAME.to_string(),
        slippage: Decimal::percent(1),
        belief_price: Decimal::percent(420),
    };

    // Prepare an AbovePegCallback msg
    let msg = ExecuteMsg::AbovePegCallback {
        details: arb_detail,
    };

    // Ensure the 'caller' is the VAULT_CONTRACT to avoid unauthorized issues
    let info = mock_info(VAULT_CONTRACT, &[]);

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    // We should have gotten 3 messages back in this case
    assert_eq!(3, res.messages.len());
    // Verify the operations happened in the order we expect.
    // For above peg, we expect first terraswap swap tx, followed by a mint
    let first_msg = res.messages[0].msg.clone();
    match first_msg {
        CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
        CosmosMsg::Custom(_t) => panic!("unexpected"),
        CosmosMsg::Wasm(_wasm_msg) => {}
        _ => panic!("unexpected"),
    }
    // Verify the second message is indeed a Market call (to Treasury or otherwise)
    let second_msg = res.messages[1].msg.clone();
    match second_msg {
        CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
        CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
        CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected"),
        _ => panic!("unexpected"),
    }

}
