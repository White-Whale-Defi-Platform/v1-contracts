use cosmwasm_std::testing::{mock_env, mock_info};
use crate::tests::mock_querier::{mock_dependencies};
use cosmwasm_std::{CosmosMsg, coins, Uint128, Api, Decimal};
use terra_cosmwasm::TerraRoute;
use crate::tests::instantiate::mock_instantiate;
use crate::msg::ExecuteMsg;
use crate::tests::common::{TEST_CREATOR, VAULT_CONTRACT};
use crate::msg::*;
use terraswap::asset::{Asset, AssetInfo};

use crate::contract::execute;
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
        slippage: Decimal::percent(1),
        belief_price: Decimal::percent(420),
    };

    // Prepare a BelowPegCallback msg
    let msg = ExecuteMsg::BelowPegCallback {
        details: arb_detail
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
        CosmosMsg::Wasm(_wasm_msg) => {},
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
        slippage: Decimal::percent(1),
        belief_price: Decimal::percent(420),
    };

    // Prepare an AbovePegCallback msg
    let msg = ExecuteMsg::AbovePegCallback {
        details: arb_detail
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
        CosmosMsg::Wasm(_wasm_msg) => {},
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