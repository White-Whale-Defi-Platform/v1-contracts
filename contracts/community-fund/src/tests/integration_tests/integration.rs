use cosmwasm_std::{Addr, Uint128};
use terra_multi_test::{App, ContractWrapper, Executor};

use white_whale::community_fund::msg::{ConfigResponse, ExecuteMsg, QueryMsg};
use white_whale::treasury::dapp_base::common_test::TEST_CREATOR;

use crate::msg::InstantiateMsg;
use crate::tests::integration_tests::common_integration::{
    get_whale_balance, init_whale_contract, mint_some_whale, mock_app,
};

fn init_fund_contract(app: &mut App, owner: Addr, whale_token_addr: &Addr) -> Addr {
    let fund_dapp_contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));

    let fund_dapp_code_id = app.store_code(fund_dapp_contract);

    let fund_init_msg = InstantiateMsg {
        whale_token_addr: whale_token_addr.to_string(),
    };

    // Init contract
    let fund_dapp_instance = app
        .instantiate_contract(
            fund_dapp_code_id,
            owner.clone(),
            &fund_init_msg,
            &[],
            "Community Fund",
            Some(TEST_CREATOR.to_string()),
        )
        .unwrap();

    fund_dapp_instance
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let whale_token_addr = init_whale_contract(&mut app, vec![]);
    let fund_dapp = init_fund_contract(&mut app, sender.clone(), &whale_token_addr);

    let res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&fund_dapp, &QueryMsg::Config {})
        .unwrap();

    // Check for whale token address in config
    assert_eq!(whale_token_addr, res.token_addr);
}

/**
 * Spending tokens
 */

#[test]
fn unsuccessful_spend_tokens_unauthorized() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let whale_token_addr = init_whale_contract(&mut app, vec![]);
    let fund_dapp = init_fund_contract(&mut app, sender.clone(), &whale_token_addr);

    app.execute_contract(
        Addr::unchecked("unauthorized"),
        fund_dapp.clone(),
        &ExecuteMsg::Spend {
            recipient: "recipient".to_string(),
            amount: Uint128::zero(),
        },
        &[],
    )
    .unwrap_err();
}

#[test]
fn unsuccessful_spend_tokens_not_enough_tokens() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let whale_token_addr = init_whale_contract(&mut app, vec![]);
    let fund_dapp = init_fund_contract(&mut app, sender.clone(), &whale_token_addr);

    app.execute_contract(
        sender,
        fund_dapp.clone(),
        &ExecuteMsg::Spend {
            recipient: "recipient".to_string(),
            amount: Uint128::from(100u64),
        },
        &[],
    )
    .unwrap_err();
}

#[test]
fn successful_spend_tokens() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let whale_token_addr = init_whale_contract(&mut app, vec![]);
    let fund_dapp = init_fund_contract(&mut app, sender.clone(), &whale_token_addr);

    mint_some_whale(
        &mut app,
        sender.clone(),
        whale_token_addr.clone(),
        Uint128::from(1000u64),
        fund_dapp.to_string(),
    );

    let fund_balance = get_whale_balance(&mut app, whale_token_addr.clone(), fund_dapp.clone());
    assert_eq!(fund_balance, Uint128::from(1000u64));

    app.execute_contract(
        sender.clone(),
        fund_dapp.clone(),
        &ExecuteMsg::Spend {
            recipient: "recipient".to_string(),
            amount: Uint128::from(100u64),
        },
        &[],
    )
    .unwrap();

    let fund_balance = get_whale_balance(&mut app, whale_token_addr.clone(), fund_dapp.clone());
    assert_eq!(fund_balance, Uint128::from(900u64));
}

/**
 * Burning tokens
 */

#[test]
fn unsuccessful_burn_tokens_unauthorized() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let whale_token_addr = init_whale_contract(&mut app, vec![]);
    let fund_dapp = init_fund_contract(&mut app, sender.clone(), &whale_token_addr);

    app.execute_contract(
        Addr::unchecked("unauthorized"),
        fund_dapp.clone(),
        &ExecuteMsg::Burn {
            amount: Uint128::zero(),
        },
        &[],
    )
    .unwrap_err();
}

#[test]
fn unsuccessful_burn_tokens_not_enough_tokens() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let whale_token_addr = init_whale_contract(&mut app, vec![]);
    let fund_dapp = init_fund_contract(&mut app, sender.clone(), &whale_token_addr);

    app.execute_contract(
        sender,
        fund_dapp.clone(),
        &ExecuteMsg::Burn {
            amount: Uint128::from(100u64),
        },
        &[],
    )
    .unwrap_err();
}

#[test]
fn successful_burn_tokens() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let whale_token_addr = init_whale_contract(&mut app, vec![]);
    let fund_dapp = init_fund_contract(&mut app, sender.clone(), &whale_token_addr);

    mint_some_whale(
        &mut app,
        sender.clone(),
        whale_token_addr.clone(),
        Uint128::from(1000u64),
        fund_dapp.to_string(),
    );

    let fund_balance = get_whale_balance(&mut app, whale_token_addr.clone(), fund_dapp.clone());
    assert_eq!(fund_balance, Uint128::from(1000u64));

    app.execute_contract(
        sender.clone(),
        fund_dapp.clone(),
        &ExecuteMsg::Burn {
            amount: Uint128::from(100u64),
        },
        &[],
    )
    .unwrap();

    let fund_balance = get_whale_balance(&mut app, whale_token_addr.clone(), fund_dapp.clone());
    assert_eq!(fund_balance, Uint128::from(900u64));
}
