use cosmwasm_std::testing::{mock_env, mock_info};
use terraswap::asset::{Asset, AssetInfo};

use white_whale::denom::LUNA_DENOM;
use white_whale::luna_vault::msg::*;

use crate::contract::execute;
use crate::error::LunaVaultError;
use crate::state::STATE;
use crate::tests::common::TEST_CREATOR;
use crate::tests::instantiate::{mock_instantiate, mock_instantiate_no_asset_info};
use crate::tests::mock_querier::mock_dependencies;

#[test]
fn unsuccessful_flashloan_not_whitelisted() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let whitelisted_contracts = STATE
        .load(deps.as_mut().storage)
        .unwrap()
        .whitelisted_contracts;
    assert_eq!(0, whitelisted_contracts.len());

    let msg = ExecuteMsg::FlashLoan {
        payload: FlashLoanPayload {
            requested_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: LUNA_DENOM.to_string(),
                },
                amount: Default::default(),
            },
            callback: Default::default(),
        },
    };
    let info = mock_info(TEST_CREATOR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::NotWhitelisted {}) => (),
        _ => panic!("Must return LunaVaultError::NotWhitelisted"),
    }
}
