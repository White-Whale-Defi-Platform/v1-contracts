use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, MessageInfo, ReplyOn, SubMsg, WasmMsg};
use cosmwasm_std::{Api, Decimal};

use crate::contract::{execute, instantiate, query};
use crate::state::{State, STATE};
use cw20::MinterResponse;

use crate::tests::common_integration::instantiate_msg as vault_msg;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use white_whale::fee::*;
use white_whale::luna_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::luna_vault::msg::*;

use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR};

use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
use crate::error::LunaVaultError;

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let info = mock_info(TEST_CREATOR, &[]);

    let state: State = STATE.load(&deps.storage).unwrap();
    // TODO: Improve
    // assert_eq!(
    //     state,
    //     State {
    //         bluna_address: deps.api.addr_validate("bluna").unwrap(),
    //         astro_lp_address: deps.api.addr_validate(&"astro".to_string()).unwrap(),
    //         memory_address: deps.api.addr_validate(&"memory".to_string()).unwrap(),
    //         whitelisted_contracts: vec![],
    //         allow_non_whitelisted: false,
    //         exchange_rate: Default::default(),
    //         total_bond_amount: Default::default(),
    //         last_index_modification: 0,
    //         prev_vault_balance: Default::default(),
    //         actual_unbonded_amount: Default::default(),
    //         last_unbonded_time: 0,
    //         last_processed_batch: 0
    //     }
    // );
    assert_eq!(
        state.bluna_address,
        deps.api.addr_validate("bluna").unwrap()
    );

    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state.whitelisted_contracts[0],
        deps.api.addr_validate(ARB_CONTRACT).unwrap(),
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

    let msg = vault_msg(
        3,
        "warchest".to_string(),
        "anchor".to_string(),
        "bluna".to_string(),
    );

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
                    name: custom_token_name,
                    symbol: custom_token_symbol,
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
