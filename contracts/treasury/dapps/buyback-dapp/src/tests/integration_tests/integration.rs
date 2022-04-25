use cosmwasm_std::{Addr, Coin, Decimal, to_binary, Uint128};
use cw20::{Cw20Contract, Cw20ExecuteMsg};

use terra_multi_test::{App, ContractWrapper, TerraApp};
use terraswap::asset::AssetInfo;
use crate::msg::{ExecuteMsg as BuyBackExecuteMsg, InstantiateMsg};
use crate::tests::integration_tests::common_integration::{
    init_contracts, mint_some_whale, mock_app,
};
use terra_multi_test::Executor;
use terraswap::pair::PoolResponse;
use white_whale::dapps::terraswap::msg::{ExecuteMsg};
use terraswap::pair::Cw20HookMsg;
use white_whale::denom::{UST_DENOM, LUNA_DENOM};
use white_whale::memory::msg as MemoryMsg;
use white_whale::treasury::dapp_base::common_test::TEST_CREATOR;
use white_whale::treasury::msg as TreasuryMsg;

use white_whale::treasury::dapp_base::msg::{BaseInstantiateMsg, BaseInstantiateMsg as TSWAPInstantiateMsg};

use super::common_integration::{whitelist_dapp, BaseContracts};
const MILLION: u64 = 1_000_000u64;

fn init_buyback_dapp(app: &mut TerraApp, owner: Addr, base_contracts: &BaseContracts) -> Addr {
    // Upload Terraswap DApp Contract
    let buyback_dapp_contract = Box::new(ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));


    let buyback_dapp_code_id = app.store_code(buyback_dapp_contract);

    let buyback_dapp_instantiate_msg = InstantiateMsg {
        base: BaseInstantiateMsg{
            trader: owner.to_string(),
            treasury_address: base_contracts.treasury.to_string(),
            memory_addr: base_contracts.memory.to_string(),
        },
        whale_vust_lp: base_contracts.vust_whale_pair.clone(),
        vust_token: base_contracts.vust.clone(),
        whale_token: base_contracts.whale.clone()
    };

    // Init contract
    let buyback_dapp_instance = app
        .instantiate_contract(
            buyback_dapp_code_id,
            owner.clone(),
            &buyback_dapp_instantiate_msg,
            &[],
            "buyback_dapp",
            None,
        )
        .unwrap();

    whitelist_dapp(app, &owner, &base_contracts.treasury, &buyback_dapp_instance);
    buyback_dapp_instance
}

fn init_terraswap_dapp(app: &mut TerraApp, owner: Addr, base_contracts: &BaseContracts) -> Addr {
    // Upload Terraswap DApp Contract
    let tswap_dapp_contract = Box::new(ContractWrapper::new_with_empty(
        terraswap_dapp::contract::execute,
        terraswap_dapp::contract::instantiate,
        terraswap_dapp::contract::query,
    ));

    let tswap_dapp_code_id = app.store_code(tswap_dapp_contract);

    let tswap_dapp_instantiate_msg = TSWAPInstantiateMsg {
        trader: owner.to_string(),
        treasury_address: base_contracts.treasury.to_string(),
        memory_addr: base_contracts.memory.to_string(),
    };

    // Init contract
    let tswap_dapp_instance = app
        .instantiate_contract(
            tswap_dapp_code_id,
            owner.clone(),
            &tswap_dapp_instantiate_msg,
            &[],
            "Tswap_dapp",
            None,
        )
        .unwrap();

    whitelist_dapp(app, &owner, &base_contracts.treasury, &tswap_dapp_instance);
    tswap_dapp_instance
}


#[test]
fn proper_initialization_and_commence_buyback() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let base_contracts = init_contracts(&mut app);
    let buyback_dapp = init_buyback_dapp(&mut app, sender.clone(), &base_contracts);
    let tswap_dapp = init_terraswap_dapp(&mut app, sender.clone(), &base_contracts);

    let resp: TreasuryMsg::ConfigResponse = app
        .wrap()
        .query_wasm_smart(&base_contracts.treasury, &TreasuryMsg::QueryMsg::Config {})
        .unwrap();

    // Check config, tswap dapp is added
    assert_eq!(2, resp.dapps.len());

    // Add whale and whale_ust token to the memory assets
    // Is tested on unit-test level
    app.execute_contract(
        sender.clone(),
        base_contracts.memory.clone(),
        &MemoryMsg::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![
                (
                    "whale".to_string(),
                    AssetInfo::Token {
                        contract_addr: base_contracts.whale.to_string(),
                    },
                ),
                (
                    "whale_ust".to_string(),
                    AssetInfo::Token {
                        contract_addr: base_contracts.whale_ust.to_string(),
                    },
                ),
                (
                    "ust".to_string(),
                    AssetInfo::NativeToken {
                        denom: UST_DENOM.to_string(),
                    },
                ),
                (
                    "luna".to_string(),
                    AssetInfo::NativeToken {
                        denom: LUNA_DENOM.to_string(),
                    },
                ),
                (
                    "vust".to_string(),
                    AssetInfo::Token {
                        contract_addr: base_contracts.vust.to_string(),
                    },
                ),

            ],
            to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Check Memory
    let resp: MemoryMsg::AssetQueryResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.memory,
            &MemoryMsg::QueryMsg::QueryAssets {
                names: vec![
                    "whale".to_string(),
                    "whale_ust".to_string(),
                    "ust".to_string(),
                    "luna".to_string(),
                    "vust".to_string(),
                ],
            },
        )
        .unwrap();

    // Detailed check handled in unit-tests
    assert_eq!("luna".to_string(), resp.assets[0].0);
    assert_eq!("ust".to_string(), resp.assets[1].0);
    assert_eq!("vust".to_string(), resp.assets[2].0);
    assert_eq!("whale".to_string(), resp.assets[3].0);
    assert_eq!("whale_ust".to_string(), resp.assets[4].0);


    // Add whale_ust pair to the memory contracts
    // Is tested on unit-test level
    app.execute_contract(
        sender.clone(),
        base_contracts.memory.clone(),
        &MemoryMsg::ExecuteMsg::UpdateContractAddresses {
            to_add: vec![(
                "vust_whale_pair".to_string(),
                base_contracts.vust_whale_pair.to_string(),
            )],
            to_remove: vec![],
        },
        &[],
    )
        .unwrap();

    // Check Memory
    let resp: MemoryMsg::ContractQueryResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.memory,
            &MemoryMsg::QueryMsg::QueryContracts {
                names: vec!["vust_whale_pair".to_string()],
            },
        )
        .unwrap();

    // Detailed check handled in unit-tests
    assert_eq!("vust_whale_pair".to_string(), resp.contracts[0].0);

    // give treasury some uusd
    app.init_bank_balance(
        &base_contracts.treasury,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u64 * MILLION),
        }],
    )
    .unwrap();

    // give treasury some whale
    mint_some_whale(
        &mut app,
        sender.clone(),
        base_contracts.whale,
        Uint128::from(10000u64 * MILLION),
        base_contracts.treasury.to_string(),
    );

    // give treasury some whale
    mint_some_whale(
        &mut app,
        sender.clone(),
        base_contracts.vust.clone(),
        Uint128::from(1001u64 * MILLION),
        base_contracts.treasury.to_string(),
    );


    // Add liquidity to pair from treasury, through terraswap-dapp
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::DetailedProvideLiquidity {
            pool_id: "vust_whale_pair".to_string(),
            assets: vec![
                ("vust".into(), Uint128::from(1000u64 * MILLION)),
                (("whale".into(), Uint128::from(1000u64 * MILLION))),
            ],
            slippage_tolerance: None,
        },
        &[],
    )
        .unwrap();

    //
    let pool_res: PoolResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.vust_whale_pair.clone(),
            &terraswap::pair::QueryMsg::Pool {},
        )
        .unwrap();

    let lp = Cw20Contract(base_contracts.vust_whale_lp.clone());

    // Get treasury lp token balance
    let treasury_bal = lp.balance(&app, base_contracts.treasury.clone()).unwrap();

    // 1 WHALE and UST in pool
    assert_eq!(Uint128::from(1000u64 * MILLION), pool_res.assets[0].amount);
    assert_eq!(Uint128::from(1000u64 * MILLION), pool_res.assets[1].amount);
    // All LP tokens owned by treasury
    assert_eq!(treasury_bal, pool_res.total_share);

    //Use BuyBack_Dapp to perform a simple buyback
    app.execute_contract(
        sender.clone(),
        buyback_dapp.clone(),
        &BuyBackExecuteMsg::Buyback {
            amount: Uint128::from(10u64),
        },
        &[],
    ).unwrap();

    app.execute_contract(
        sender.clone(),
        buyback_dapp.clone(),
        &BuyBackExecuteMsg::Buyback {
            amount: Uint128::from(1000u64),
        },
        &[],
    ).unwrap();

    // Lets try a buyback with too much funds
    app.execute_contract(
        sender.clone(),
        buyback_dapp.clone(),
        &BuyBackExecuteMsg::Buyback {
            amount: Uint128::from(10000000000u64),
        },
        &[],
    ).unwrap_err();
}
