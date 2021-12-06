use crate::contract::{execute, instantiate};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Uint128};
use terraswap::asset::{AssetInfo, Asset};
use white_whale::treasury::msg::{ExecuteMsg, InstantiateMsg};
use crate::tests::common::TEST_CREATOR;
use white_whale::treasury::vault_assets::*;

fn init_msg() -> InstantiateMsg {
    InstantiateMsg {}
}

#[test]
fn test_send_token() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddTrader {
        trader: TEST_CREATOR.to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }

    let test_token_asset = VaultAsset{
        asset: Asset {
            info: AssetInfo::Token{
                contract_addr: "test_token".to_string()
            },
            amount: Uint128::zero()
        },
        value_reference: None
    };

    let msg = ExecuteMsg::UpdateAssets {
        to_add: vec![test_token_asset.clone()],
        to_remove: vec![],
    };

    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();


    let msg = ExecuteMsg::SendAsset {
        id: get_identifier(&test_token_asset.asset.info).clone(),
        amount: Uint128::from(10000u64),
        recipient: TEST_CREATOR.to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(res) => {
            assert_eq!(res.messages.len(), 1); 
        },
        Err(e) => panic!("{}", e),
    }
}
