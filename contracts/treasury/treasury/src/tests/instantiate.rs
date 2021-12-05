use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, DepsMut, MessageInfo, ReplyOn, SubMsg, WasmMsg};
use cosmwasm_std::{Api, Decimal, Uint128};

use crate::contract::{execute, instantiate, query};

use crate::error::TreasuryError;
use terraswap::asset::{AssetInfo, Asset};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use white_whale::fee::*;
use white_whale::treasury::msg::*;
use white_whale::treasury::state::*;
use white_whale::treasury::vault_assets::*;

use crate::tests::common::{DAPP, TEST_CREATOR};

const INSTANTIATE_REPLY_ID: u8 = 1u8;
pub(crate) const WARCHEST_FEE: u64 = 10u64;

pub fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {}
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let msg = InstantiateMsg {};

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, msg).expect("Contract failed init");
}

/**
 * Tests successful instantiation of the contract. 
 * Addition of a trader
 * Removal of a trader
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    // Response should have 0 msgs
    assert_eq!(0, res.messages.len());

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            traders: vec![],
        }
    );

    let msg = ExecuteMsg::AddTrader {
        trader: DAPP.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state.traders[0],
        deps.api.addr_canonicalize(&DAPP).unwrap(),
    );

    let msg = ExecuteMsg::RemoveTrader {
        trader: DAPP.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            traders: vec![],
        }
    );
}

/**
 * Tests successful Vault Asset update
 */
#[test]
fn successful_asset_update() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    // Response should have 0 msgs
    assert_eq!(0, res.messages.len());

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            traders: vec![],
        }
    );

    let test_native_asset = VaultAsset{
        asset: Asset {
            info: AssetInfo::NativeToken{
                denom: "base_asset".to_string()
            },
            amount: Uint128::zero()
        },
        value_reference: None
    };

    let test_token_asset= VaultAsset{
        asset: Asset {
            info: AssetInfo::Token{
                contract_addr: "test_token".to_string()
            },
            amount: Uint128::zero()
        },
        value_reference: None
    };


    let msg = ExecuteMsg::UpdateAssets {
        to_add: vec![test_native_asset.clone(),test_token_asset.clone()],
        to_remove: vec![],
    };

    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Get an asset
    let asset_1: VaultAsset = VAULT_ASSETS.load(&deps.storage, get_identifier(&test_native_asset.asset.info)).unwrap();
    assert_eq!(
        test_native_asset,
        asset_1,
    );
    // Get the other asset
    let asset_2: VaultAsset = VAULT_ASSETS.load(&deps.storage, get_identifier(&test_token_asset.asset.info)).unwrap();
    assert_eq!(
        test_token_asset,
        asset_2,
    );

    // Remove token 
    let msg = ExecuteMsg::UpdateAssets {
        to_add: vec![],
        to_remove: vec![test_token_asset.asset.info.clone()],
    };

    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let _failed_load = VAULT_ASSETS.load(&deps.storage, get_identifier(&test_token_asset.asset.info)).unwrap_err();
}


// /**
//  * Tests updating the fees of the contract.
//  */
// #[test]
// fn successful_update_fee() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());

//     // update fees
//     let info = mock_info(TEST_CREATOR, &[]);
//     let msg = ExecuteMsg::SetFee {
//         flash_loan_fee: Some(Fee {
//             share: Decimal::percent(1),
//         }),
//         warchest_fee: Some(Fee {
//             share: Decimal::percent(2),
//         }),
//     };

//     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//     assert_eq!(0, res.messages.len());

//     // it worked, let's query the fee
//     let res = query(deps.as_ref(), mock_env(), QueryMsg::Fees {}).unwrap();
//     let fee_response: FeeResponse = from_binary(&res).unwrap();
//     let fees: VaultFee = fee_response.fees;
//     assert_eq!(Decimal::percent(1), fees.flash_loan_fee.share);
//     assert_eq!(Decimal::percent(2), fees.warchest_fee.share);
// }

// #[test]
// fn unsuccessful_update_fee_unauthorized() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());

//     // update fees
//     let info = mock_info("unauthorized", &[]);
//     let msg = ExecuteMsg::SetFee {
//         flash_loan_fee: Some(Fee {
//             share: Decimal::percent(1),
//         }),
//         warchest_fee: Some(Fee {
//             share: Decimal::percent(2),
//         }),
//     };

//     let res = execute(deps.as_mut(), mock_env(), info, msg);
//     match res {
//         Err(TreasuryError::Admin(_)) => (),
//         _ => panic!("Must return TreasuryError::Admin"),
//     }
// }

// #[test]
// fn successful_update_fee_unchanged() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());

//     let fees = FEE.load(deps.as_mut().storage).unwrap();
//     let original_flash_loan_fee = fees.flash_loan_fee;
//     let original_warchest_fee = fees.warchest_fee;

//     // update fees
//     let info = mock_info(TEST_CREATOR, &[]);
//     let msg = ExecuteMsg::SetFee {
//         flash_loan_fee: None,
//         warchest_fee: None,
//     };

//     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//     assert_eq!(0, res.messages.len());

//     let res = query(deps.as_ref(), mock_env(), QueryMsg::Fees {}).unwrap();
//     let fee_response: FeeResponse = from_binary(&res).unwrap();
//     let fees: VaultFee = fee_response.fees;
//     assert_eq!(original_flash_loan_fee.share, fees.flash_loan_fee.share);
//     assert_eq!(original_warchest_fee.share, fees.warchest_fee.share);
// }

// #[test]
// fn successfull_set_admin() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());

//     // update admin
//     let info = mock_info(TEST_CREATOR, &[]);
//     let msg = ExecuteMsg::SetAdmin {
//         admin: "new_admin".to_string(),
//     };

//     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//     assert_eq!(0, res.messages.len());
// }

// #[test]
// fn unsuccessfull_set_admin_unauthorized() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());

//     // update admin
//     let info = mock_info("unauthorized", &[]);
//     let msg = ExecuteMsg::SetAdmin {
//         admin: "new_admin".to_string(),
//     };

//     let res = execute(deps.as_mut(), mock_env(), info, msg);
//     match res {
//         Err(TreasuryError::Admin(_)) => (),
//         _ => panic!("Must return TreasuryError::Admin"),
//     }
// }

// #[test]
// fn test_init_with_non_default_vault_lp_token() {
//     let mut deps = mock_dependencies(&[]);

//     let custom_token_name = String::from("My LP Token");
//     let custom_token_symbol = String::from("MyLP");

//     // Define a custom Init Msg with the custom token info provided
//     let msg = InstantiateMsg {
//         anchor_money_market_address: "test_mm".to_string(),
//         aust_address: "test_aust".to_string(),
//         profit_check_address: "test_profit_check".to_string(),
//         warchest_addr: "warchest".to_string(),
//         asset_info: AssetInfo::NativeToken {
//             denom: "uusd".to_string(),
//         },
//         token_code_id: 10u64,
//         warchest_fee: Decimal::percent(10u64),
//         flash_loan_fee: Decimal::permille(5u64),
//         stable_cap: Uint128::from(1000_000_000u64),
//         vault_lp_token_name: Some(custom_token_name.clone()),
//         vault_lp_token_symbol: Some(custom_token_symbol.clone()),
//     };

//     // Prepare mock env
//     let env = mock_env();
//     let info = MessageInfo {
//         sender: deps.api.addr_validate("creator").unwrap(),
//         funds: vec![],
//     };

//     let res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
//     // Ensure we have 1 message
//     assert_eq!(1, res.messages.len());
//     // Verify the message is the one we expect but also that our custom provided token name and symbol were taken into account.
//     assert_eq!(
//         res.messages,
//         vec![SubMsg {
//             // Create LP token
//             msg: WasmMsg::Instantiate {
//                 admin: None,
//                 code_id: msg.token_code_id,
//                 msg: to_binary(&TokenInstantiateMsg {
//                     name: custom_token_name.to_string(),
//                     symbol: custom_token_symbol.to_string(),
//                     decimals: 6,
//                     initial_balances: vec![],
//                     mint: Some(MinterResponse {
//                         minter: env.contract.address.to_string(),
//                         cap: None,
//                     }),
//                 })
//                 .unwrap(),
//                 funds: vec![],
//                 label: "White Whale Stablecoin Vault LP".to_string(),
//             }
//             .into(),
//             gas_limit: None,
//             id: u64::from(INSTANTIATE_REPLY_ID),
//             reply_on: ReplyOn::Success,
//         }]
//     );
// }
