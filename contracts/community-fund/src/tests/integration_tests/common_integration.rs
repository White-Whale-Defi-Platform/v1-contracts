use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Addr, Empty, Uint128};
use cw20::{BalanceResponse, Cw20Coin, Cw20QueryMsg};
use terra_mocks::TerraMockQuerier;
use terra_multi_test::{App, BankKeeper, ContractWrapper, Executor};

use white_whale::treasury::dapp_base::common_test::TEST_CREATOR;

/// Instantiates the whale token contract
pub fn init_whale_contract(app: &mut App, initial_balances: Vec<Cw20Coin>) -> Addr {
    let owner = Addr::unchecked(TEST_CREATOR);

    // Instantiate WHALE Token Contract
    let cw20_token_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let cw20_token_code_id = app.store_code(cw20_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Whale token"),
        symbol: String::from("WHALE"),
        decimals: 6,
        initial_balances,
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let whale_token_instance = app
        .instantiate_contract(cw20_token_code_id, owner.clone(), &msg, &[], "WHALE", None)
        .unwrap();

    whale_token_instance
}

pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let custom_querier: TerraMockQuerier =
        TerraMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    App::new(api, env.block, bank, MockStorage::new(), custom_querier)
}

/// Mint Whale tokens
pub fn mint_some_whale(
    app: &mut App,
    owner: Addr,
    whale_token_instance: Addr,
    amount: Uint128,
    to: String,
) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: to.clone(),
        amount,
    };
    let res = app
        .execute_contract(owner.clone(), whale_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", to));
    assert_eq!(res.events[1].attributes[3], attr("amount", amount));
}

/// check whale balance
pub fn get_whale_balance(app: &mut App, whale_token_instance: Addr, address: Addr) -> Uint128 {
    let res: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            whale_token_instance,
            &Cw20QueryMsg::Balance {
                address: address.to_string(),
            },
        )
        .unwrap();

    res.balance
}
