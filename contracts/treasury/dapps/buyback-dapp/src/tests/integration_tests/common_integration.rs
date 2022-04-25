use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Addr, Empty, Timestamp, Uint128};
use schemars::_serde_json::to_string;
use terra_mocks::TerraMockQuerier;
use terra_multi_test::{App, AppBuilder, BankKeeper, ContractWrapper, Executor, TerraMock, TerraApp};
use terraswap::asset::{AssetInfo, PairInfo};
use white_whale::memory::msg as MemoryMsg;
use white_whale::treasury::dapp_base::common_test::TEST_CREATOR;
use white_whale::treasury::msg as TreasuryMsg;

#[allow(dead_code)]
pub struct BaseContracts {
    pub whale: Addr,
    pub memory: Addr,
    pub treasury: Addr,
    pub whale_ust_pair: Addr,
    pub whale_ust: Addr,
    pub vust: Addr,
    pub vust_whale_pair: Addr,
}

#[allow(dead_code)]
/// Creates the basic contract instances needed to test the dapp.
/// Whale token, Memory, Treasury, Whale/UST pair, Whale/UST LP
pub fn init_contracts(app: &mut TerraApp) -> BaseContracts {
    let owner = Addr::unchecked(TEST_CREATOR);

    // Instantiate WHALE Token Contract
    let cw20_token_contract = Box::new(ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let cw20_token_code_id = app.store_code(cw20_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Whale token"),
        symbol: String::from("WHALE"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let whale_token_instance = app
        .instantiate_contract(
            cw20_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("WHALE"),
            None,
        )
        .unwrap();

    let vust_msg = cw20_base::msg::InstantiateMsg {
        name: String::from("vUST Token"),
        symbol: String::from("VUST"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let vust_token_instance = app
        .instantiate_contract(
            cw20_token_code_id,
            owner.clone(),
            &vust_msg,
            &[],
            String::from("VUST"),
            None,
        )
        .unwrap();

    // Upload Treasury Contract
    let treasury_contract = Box::new(ContractWrapper::new_with_empty(
        treasury::contract::execute,
        treasury::contract::instantiate,
        treasury::contract::query,
    ));

    let treasury_code_id = app.store_code(treasury_contract);

    let treasury_instantiate_msg = TreasuryMsg::InstantiateMsg {};

    // Instantiate Treasury Contract
    let treasury_instance = app
        .instantiate_contract(
            treasury_code_id,
            owner.clone(),
            &treasury_instantiate_msg,
            &[],
            "Treasury",
            None,
        )
        .unwrap();

    // Upload Memory Contract
    let memory_contract = Box::new(ContractWrapper::new_with_empty(
        memory::contract::execute,
        memory::contract::instantiate,
        memory::contract::query,
    ));

    let memory_code_id = app.store_code(memory_contract);

    let memory_instantiate_msg = MemoryMsg::InstantiateMsg {};

    // Init contract
    let memory_instance = app
        .instantiate_contract(
            memory_code_id,
            owner.clone(),
            &memory_instantiate_msg,
            &[],
            "Memory",
            None,
        )
        .unwrap();

    // Instantiate the terraswap pair
    let (pair, lp) = instantiate_pair(app, &owner.clone(), &whale_token_instance);
    // Instantiate the terraswap pair
    let (vustpair, vustlp) = instantiate_vust_whale_pair(app, &owner.clone(), &whale_token_instance, &vust_token_instance);

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571797419);
    });

    BaseContracts {
        treasury: treasury_instance,
        memory: memory_instance,
        whale: whale_token_instance,
        whale_ust_pair: pair,
        whale_ust: lp,
        vust: vust_token_instance,
        vust_whale_pair: vustpair
    }
}

#[allow(dead_code)]
pub fn mock_app() -> TerraApp {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();
    let custom = TerraMock::luna_ust_case();

    // TODO: and NOTE I removed the custom querier here and updated deps, custom_querier most likely needs to be added again
    // let custom_querier: TerraMockQuerier =
    //     TerraMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    // App::new(api, env.block, bank, MockStorage::new(), custom_querier)
    // let custom_handler = CachingCustomHandler::<CustomMsg, Empty>::new();
    // AppBuilder::new().with_custom(custom_handler).build()
    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .with_custom(custom)
        .build()
}

/// Create terraswap WHALE/UST pair
fn instantiate_pair(
    mut router: &mut TerraApp,
    owner: &Addr,
    whale_token_instance: &Addr
) -> (Addr, Addr) {
    let token_contract_code_id = store_token_code(&mut router);

    let pair_contract_code_id = store_pair_code(&mut router);

    let msg = terraswap::pair::InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: whale_token_instance.to_string(),
            },
        ],
        token_code_id: token_contract_code_id,
    };

    let pair = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIRRR"),
            None,
        )
        .unwrap();

    let res: PairInfo = router
        .wrap()
        .query_wasm_smart(pair.clone(), &terraswap::pair::QueryMsg::Pair {})
        .unwrap();

    (pair, Addr::unchecked(res.liquidity_token))
}

/// Create terraswap WHALE/UST pair
fn instantiate_vust_whale_pair(
    mut router: &mut TerraApp,
    owner: &Addr,
    whale_token_instance: &Addr,
    base_token_instance: &Addr
) -> (Addr, Addr) {
    let token_contract_code_id = store_token_code(&mut router);

    let pair_contract_code_id = store_pair_code(&mut router);

    let msg = terraswap::pair::InstantiateMsg {
        asset_infos: [
            AssetInfo::Token {
                contract_addr: base_token_instance.to_string(),
            },
            AssetInfo::Token {
                contract_addr: whale_token_instance.to_string(),
            },
        ],
        token_code_id: token_contract_code_id,
    };

    let pair = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIRRR"),
            None,
        )
        .unwrap();

    let res: PairInfo = router
        .wrap()
        .query_wasm_smart(pair.clone(), &terraswap::pair::QueryMsg::Pair {})
        .unwrap();

    (pair, Addr::unchecked(res.liquidity_token))
}

/// Whitelist a dapp on the treasury
#[allow(dead_code)]
pub fn whitelist_dapp(app: &mut TerraApp, owner: &Addr, treasury_instance: &Addr, dapp_instance: &Addr) {
    let msg = TreasuryMsg::ExecuteMsg::AddDApp {
        dapp: dapp_instance.to_string(),
    };
    let _res = app
        .execute_contract(owner.clone(), treasury_instance.clone(), &msg, &[])
        .unwrap();
    // Check if it was added
    let resp: TreasuryMsg::ConfigResponse = app
        .wrap()
        .query_wasm_smart(treasury_instance, &TreasuryMsg::QueryMsg::Config {})
        .unwrap();

    // Check config
    assert!(resp.dapps.contains(&dapp_instance.to_string()));
}

/// Mint Whale tokens
#[allow(dead_code)]
pub fn mint_some_whale(
    app: &mut TerraApp,
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

fn store_token_code(app: &mut TerraApp) -> u64 {
    let whale_token_contract = Box::new(ContractWrapper::new_with_empty(
        terraswap_token::contract::execute,
        terraswap_token::contract::instantiate,
        terraswap_token::contract::query,
    ));

    app.store_code(whale_token_contract)
}

fn store_pair_code(app: &mut TerraApp) -> u64 {
    let pair_contract = Box::new(
        ContractWrapper::new_with_empty(
            terraswap_pair::contract::execute,
            terraswap_pair::contract::instantiate,
            terraswap_pair::contract::query,
        )
        .with_reply_empty(terraswap_pair::contract::reply),
    );

    app.store_code(pair_contract)
}

#[allow(dead_code)]
fn store_factory_code(app: &mut TerraApp) -> u64 {
    let factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            terraswap_factory::contract::execute,
            terraswap_factory::contract::instantiate,
            terraswap_factory::contract::query,
        )
        .with_reply_empty(terraswap_factory::contract::reply),
    );

    app.store_code(factory_contract)
}
