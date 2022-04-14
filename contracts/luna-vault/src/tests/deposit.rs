use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, MessageInfo, ReplyOn, SubMsg, WasmMsg};
use cosmwasm_std::{Api, Decimal, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::state::{State, STATE};
use cw20::MinterResponse;

use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use white_whale::fee::*;
use white_whale::ust_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::ust_vault::msg::*;

use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR};

use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
use crate::error::LunaVaultError;
use terraswap::asset::AssetInfo;

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let info = mock_info(TEST_CREATOR, &[]);

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            anchor_money_market_address: deps.api.addr_validate("test_mm").unwrap(),
            bluna_address: deps.api.addr_validate("test_aust").unwrap(),
            whitelisted_contracts: vec![],
            allow_non_whitelisted: false
        }
    );

    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state.whitelisted_contracts[0],
        deps.api.addr_validate(&ARB_CONTRACT).unwrap(),
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
        treasury_fee: Some(Fee {
            share: Decimal::percent(2),
        }),
        flash_loan_fee: Some(Fee {
            share: Decimal::percent(2),
        }),
        commission_fee: Some(Fee {
            share: Decimal::percent(2),
        }),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the fee
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Fees {}).unwrap();
    let fee_response: FeeResponse = from_binary(&res).unwrap();
    let fees: VaultFee = fee_response.fees;
    assert_eq!(Decimal::percent(2), fees.treasury_fee.share);
    assert_eq!(Decimal::percent(2), fees.commission_fee.share);
    assert_eq!(Decimal::percent(2), fees.flash_loan_fee.share);
}

#[test]
fn sad_path_update_fee() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update fees with an amount that should fail validation
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::SetFee {
        treasury_fee: Some(Fee {
            share: Decimal::percent(200),
        }),
        flash_loan_fee: Some(Fee {
            share: Decimal::percent(200),
        }),
        commission_fee: Some(Fee {
            share: Decimal::percent(200),
        }),
    };
    // Also test with exactly 100. We cant set fees as 100 otherwise theres nothing but fees
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    match res {
        LunaVaultError::InvalidFee {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
    let msg = ExecuteMsg::SetFee {
        treasury_fee: Some(Fee {
            share: Decimal::percent(100),
        }),
        flash_loan_fee: Some(Fee {
            share: Decimal::percent(100),
        }),
        commission_fee: Some(Fee {
            share: Decimal::percent(100),
        }),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match res {
        LunaVaultError::InvalidFee {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn successfull_set_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update admin
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::SetAdmin {
        admin: "new_admin".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_init_with_non_default_vault_lp_token() {
    let mut deps = mock_dependencies(&[]);

    let custom_token_name = String::from("My LP Token");
    let custom_token_symbol = String::from("MyLP");

    let msg = InstantiateMsg {
        anchor_money_market_address: "test_mm".to_string(),
        bluna_address: "test_aust".to_string(),
        treasury_addr: "treasury".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: 0u64,
        treasury_fee: Decimal::percent(10u64),
        flash_loan_fee: Decimal::permille(5u64),
        commission_fee: Decimal::permille(8u64),
        stable_cap: Uint128::from(100_000_000u64),
        vault_lp_token_name: Some(custom_token_name.clone()),
        vault_lp_token_symbol: Some(custom_token_symbol.clone()),
    };

    // Prepare mock env
    let env = mock_env();
    let info = MessageInfo {
        sender: deps.api.addr_validate("creator").unwrap(),
        funds: vec![],
    };

    let res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
    // Ensure we have 1 message
    assert_eq!(1, res.messages.len());
    // Verify the message is the one we expect but also that our custom provided token name and symbol were taken into account.
    assert_eq!(
        res.messages,
        vec![SubMsg {
            // Create LP token
            msg: WasmMsg::Instantiate {
                admin: None,
                code_id: msg.token_code_id,
                msg: to_binary(&TokenInstantiateMsg {
                    name: custom_token_name.to_string(),
                    symbol: custom_token_symbol.to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                })
                .unwrap(),
                funds: vec![],
                label: "White Whale Stablecoin Vault LP".to_string(),
            }
            .into(),
            gas_limit: None,
            id: u64::from(INSTANTIATE_REPLY_ID),
            reply_on: ReplyOn::Success,
        }]
    );
}
