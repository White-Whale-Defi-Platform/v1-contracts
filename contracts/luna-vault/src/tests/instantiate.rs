use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Addr, DepsMut, MessageInfo, ReplyOn, SubMsg, WasmMsg};
use cosmwasm_std::{Api, Decimal, Uint128};
use cw20::MinterResponse;
use terraswap::asset::AssetInfo;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use crate::tests::mock_querier::{mock_dependencies};
use white_whale::fee::*;
use white_whale::luna_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::luna_vault::msg::*;

use crate::contract::{execute, instantiate, query};
use crate::error::LunaVaultError;
use crate::state::{State, FEE, STATE};
use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR};
use crate::tests::common_integration::{instantiate_msg as vault_msg};
use white_whale::luna_vault::msg::InstantiateMsg as VaultInstantiateMsg;

use astroport::asset::PairInfo;
use astroport::factory::PairType;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
pub(crate) const TREASURY_FEE: u64 = 10u64;

pub fn instantiate_msg() -> InstantiateMsg {
    vault_msg(
        3,
        "warchest".to_string(),
        "astro".to_string(),
        "bluna".to_string(),
        "cluna".to_string(),
    )
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let msg = vault_msg(
        3,
        "warchest".to_string(),
        "astro".to_string(),
        "bluna".to_string(),
        "cluna".to_string(),
    );

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, msg).expect("Contract failed init");
}

pub fn mock_instantiate_no_asset_info(deps: DepsMut) {
    let bluna_address = "bluna".to_string();
    let msg = InstantiateMsg {
        bluna_address: bluna_address.clone(),
        cluna_address: bluna_address.clone(),
        astro_lp_address: bluna_address,
        astro_factory_address: "astro_factory".to_string(),
        treasury_addr: "war_chest".to_string(),
        memory_addr: "memory".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "".to_string(),
        },
        token_code_id: 3,
        treasury_fee: Decimal::percent(10u64),
        flash_loan_fee: Decimal::permille(5u64),
        commission_fee: Decimal::permille(8u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
        unbond_handler_code_id: 0,
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, msg).expect("Contract failed init");
}

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_astroport_pairs(&[(
        &"astro".to_string(),
        &PairInfo {
            asset_infos: [
                astroport::asset::AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0000"),
                },
                astroport::asset::AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
            ],
            contract_addr: Addr::unchecked("pair0000"),
            liquidity_token: Addr::unchecked("liquidity0000"),
            pair_type: PairType::Xyk {},
        },
    )]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    // Response should have one Msg for creating the LP token
    assert_eq!(1, res.messages.len());

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            bluna_address: deps.api.addr_validate("bluna").unwrap(),
            cluna_address: deps.api.addr_validate("cluna").unwrap(),
            astro_lp_address: deps.api.addr_validate("astro").unwrap(),
            astro_factory_address: Addr::unchecked("astro_factory_address".to_string()),
            memory_address: deps.api.addr_validate("memory").unwrap(),
            whitelisted_contracts: vec![],
            allow_non_whitelisted: false,
            unbond_handler_code_id: 0,
        }
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

#[test]
fn unsuccessful_initialization_invalid_fees() {
    let mut deps = mock_dependencies(&[]);



    let msg = VaultInstantiateMsg {
        bluna_address: "bluna".to_string(),
        cluna_address: "cluna".to_string(),
        astro_lp_address: "astro".to_string(),
        astro_factory_address: "astro_factory_address".to_string(),
        treasury_addr: "warchest".to_string(),
        memory_addr: "memory".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uluna".to_string(),
        },
        token_code_id: 3,
        treasury_fee: Decimal::percent(10000u64),
        flash_loan_fee: Decimal::permille(5000u64),
        commission_fee: Decimal::permille(8000u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
        unbond_handler_code_id: 0,
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    println!("{:?}",res);
    match res {
        Err(LunaVaultError::InvalidFee {}) => (),
        _ => panic!("Must return LunaVaultError::InvalidFee"),
    }
}

#[test]
fn unsuccessful_initialization_invalid_asset() {
    let mut deps = mock_dependencies(&[]);
    let bluna_address = "bluna".to_string();
    let msg = InstantiateMsg {
        bluna_address: bluna_address.clone(),
        cluna_address: bluna_address.clone(),
        astro_lp_address: bluna_address,
        astro_factory_address: "astro_factory".to_string(),
        treasury_addr: "war_chest".to_string(),
        memory_addr: "memory".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: 3,
        treasury_fee: Decimal::percent(10u64),
        flash_loan_fee: Decimal::permille(5u64),
        commission_fee: Decimal::permille(8u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
        unbond_handler_code_id: 0,
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::NotLunaToken {}) => (),
        _ => panic!("Must return LunaVaultError::NotLunaToken"),
    }
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
        flash_loan_fee: Some(Fee {
            share: Decimal::percent(1),
        }),
        treasury_fee: Some(Fee {
            share: Decimal::percent(2),
        }),
        commission_fee: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the fee
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Fees {}).unwrap();
    let fee_response: FeeResponse = from_binary(&res).unwrap();
    let fees: VaultFee = fee_response.fees;
    assert_eq!(Decimal::percent(1), fees.flash_loan_fee.share);
    assert_eq!(Decimal::percent(2), fees.treasury_fee.share);
}

#[test]
fn unsuccessful_update_fee_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update fees
    let info = mock_info("unauthorized", &[]);
    let msg = ExecuteMsg::SetFee {
        flash_loan_fee: Some(Fee {
            share: Decimal::percent(1),
        }),
        treasury_fee: Some(Fee {
            share: Decimal::percent(2),
        }),
        commission_fee: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}

#[test]
fn successful_update_fee_unchanged() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let fees = FEE.load(deps.as_mut().storage).unwrap();
    let original_flash_loan_fee = fees.flash_loan_fee;
    let original_treasury_fee = fees.treasury_fee;

    // update fees
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::SetFee {
        flash_loan_fee: None,
        treasury_fee: None,
        commission_fee: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Fees {}).unwrap();
    let fee_response: FeeResponse = from_binary(&res).unwrap();
    let fees: VaultFee = fee_response.fees;
    assert_eq!(original_flash_loan_fee.share, fees.flash_loan_fee.share);
    assert_eq!(original_treasury_fee.share, fees.treasury_fee.share);
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
fn unsuccessful_set_admin_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update admin
    let info = mock_info("unauthorized", &[]);
    let msg = ExecuteMsg::SetAdmin {
        admin: "new_admin".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}

#[test]
fn test_init_with_non_default_vault_lp_token() {
    let mut deps = mock_dependencies(&[]);

    let custom_token_name = String::from("My LP Token");
    let custom_token_symbol = String::from("MyLP");

    // Define a custom Init Msg with the custom token info provided
    let msg = VaultInstantiateMsg {
        bluna_address: "bluna".to_string(),
        cluna_address: "cluna".to_string(),
        astro_lp_address: "astro".to_string(),
        astro_factory_address: "astro_factory_address".to_string(),
        treasury_addr: "warchest".to_string(),
        memory_addr: "memory".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uluna".to_string(),
        },
        token_code_id: 3,
        treasury_fee: Decimal::percent(1u64),
        flash_loan_fee: Decimal::permille(5u64),
        commission_fee: Decimal::permille(8u64),
        vault_lp_token_name: Some(custom_token_name.clone()),
        vault_lp_token_symbol: Some(custom_token_symbol.clone()),
        unbond_handler_code_id: 0,
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
                label: "White Whale Luna Vault LP".to_string(),
            }
            .into(),
            gas_limit: None,
            id: u64::from(INSTANTIATE_REPLY_ID),
            reply_on: ReplyOn::Success,
        }]
    );
}
