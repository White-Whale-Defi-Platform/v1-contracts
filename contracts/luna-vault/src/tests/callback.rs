use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Uint128;

use white_whale::luna_vault::msg::{CallbackMsg, ExecuteMsg};

use crate::contract::execute;
use crate::error::LunaVaultError;
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
fn unsuccessful_handle_callback_not_same_contract() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::Callback(CallbackMsg::AfterTrade {
        loan_fee: Uint128::zero(),
    });
    let info = mock_info("anything", &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::NotCallback {}) => (),
        _ => panic!("Must return StableVaultError::NotCallback"),
    }
}
