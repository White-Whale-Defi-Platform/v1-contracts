use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::DepsMut;
use cosmwasm_std::{Api, Uint128};

use crate::contract::{execute, instantiate};

use terraswap::asset::{AssetInfo, Asset};
use white_whale::treasury::msg::*;
use white_whale::treasury::state::*;
use white_whale::treasury::vault_assets::*;

use crate::tests::common::{DAPP, TEST_CREATOR};

pub fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {}
}

/**
 * Mocks instantiation.
 */
pub fn _mock_instantiate(deps: DepsMut) {
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