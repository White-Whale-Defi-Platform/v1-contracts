use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coin, Uint128};
use white_whale::denom::UST_DENOM;

use white_whale::ust_vault::msg::{CallbackMsg, ExecuteMsg};

use crate::contract::execute;
use crate::error::LunaVaultError;
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
fn unsuccessful_handle_callback_not_same_contract() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::Callback {
        0: CallbackMsg::AfterTrade {
            loan_fee: Uint128::zero(),
        },
    };
    let info = mock_info("anything", &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::NotCallback {}) => (),
        _ => panic!("Must return StableVaultError::NotCallback"),
    }
}

#[test]
fn successful_handle_callback_without_anchor_deposit() {
    let mut deps = mock_dependencies(&[coin(100u128, UST_DENOM)]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::Callback {
        0: CallbackMsg::AfterTrade {
            loan_fee: Uint128::zero(),
        },
    };
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    // 1 msg (commission)
    assert_eq!(1, res.messages.len());
}

#[test]
fn successful_handle_callback_with_anchor_deposit() {
    let mut deps = mock_dependencies(&[coin(150000001u128, UST_DENOM)]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::Callback {
        0: CallbackMsg::AfterTrade {
            loan_fee: Uint128::zero(),
        },
    };
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    // 2msgs, anchor and commission
    assert_eq!(2, res.messages.len());
}
