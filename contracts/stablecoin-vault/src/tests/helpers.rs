use cosmwasm_std::{Api, CanonicalAddr, CosmosMsg, from_binary, ReplyOn, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_env, mock_info};
use cw20::Cw20ReceiveMsg;
use schemars::_private::NoSerialize;
use terraswap::asset::{AssetInfo, AssetInfoRaw};
use terraswap::pair::Cw20HookMsg;

use white_whale::profit_check::msg::ExecuteMsg as ProfitCheckMsg;
use white_whale::ust_vault::msg::CallbackMsg;

use crate::contract::{encapsulate_payload, receive_cw20};
use crate::error::StableVaultError;
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::state::{POOL_INFO, STATE};
use crate::tests::common::TEST_CREATOR;
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
fn test_encapsulate_payload() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let response = Response::new();
    let loan_fee = Uint128::new(1000);

    let res = encapsulate_payload(deps.as_ref(), mock_env(), response, loan_fee).unwrap();
    assert_eq!(res.messages.len(), 3);

    let state = STATE.load(deps.as_mut().storage).unwrap();
    let env = mock_env();

    assert_eq!(
        res.messages,
        vec![
            SubMsg {
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: deps
                        .api
                        .addr_humanize(&state.profit_check_address).unwrap()
                        .to_string(),
                    msg: to_binary(&ProfitCheckMsg::BeforeTrade {}).unwrap(),
                    funds: vec![],
                }),
            },
            SubMsg {
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
                msg: CallbackMsg::AfterSuccessfulLoanCallback {}.to_cosmos_msg(&env.contract.address).unwrap(),
            },
            SubMsg {
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: deps
                        .api
                        .addr_humanize(&state.profit_check_address).unwrap()
                        .to_string(),
                    msg: to_binary(&ProfitCheckMsg::AfterTrade { loan_fee }).unwrap(),
                    funds: vec![],
                }),
            },
        ]
    );
}

#[test]
fn unsuccessful_receive_cw20_no_swap_available() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let info = mock_info(TEST_CREATOR, &[]);

    let cw20_msg = Cw20ReceiveMsg {
        sender: "".to_string(),
        amount: Default::default(),
        msg: to_binary(&Cw20HookMsg::Swap {
            belief_price: None,
            max_spread: None,
            to: None,
        }).unwrap(),
    };

    let res = receive_cw20(deps.as_mut(), mock_env(), info, cw20_msg);
    match res {
        Err(StableVaultError::NoSwapAvailable {}) => (),
        _ => panic!("Must return StableVaultError::NoSwapAvailable"),
    }
}

#[test]
fn unsuccessful_receive_cw20_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let info = mock_info(TEST_CREATOR, &[]);

    let cw20_msg = Cw20ReceiveMsg {
        sender: "unauthorized".to_string(),
        amount: Default::default(),
        msg: to_binary(&Cw20HookMsg::WithdrawLiquidity {}).unwrap(),
    };

    let res = receive_cw20(deps.as_mut(), mock_env(), info, cw20_msg);
    match res {
        Err(StableVaultError::Unauthorized {}) => (),
        _ => panic!("Must return StableVaultError::Unauthorized"),
    }
}
