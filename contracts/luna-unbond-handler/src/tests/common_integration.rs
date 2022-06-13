use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Addr, Decimal, Empty};
use cw20::Cw20ExecuteMsg;
use terra_mocks::TerraMockQuerier;
use terra_multi_test::{
    App, AppBuilder, BankKeeper, Contract, ContractWrapper, Executor, TerraApp,
};
use terraswap::asset::AssetInfo;

use white_whale::luna_vault::luna_unbond_handler::msg as UnbondHandlerMsg;
use white_whale::luna_vault::msg as LunaVaultMsg;
use white_whale::memory::msg as MemoryMsg;
use white_whale::treasury::msg as TreasuryMsg;
use white_whale::treasury::state::LUNA_DENOM;
use white_whale::{anchor, treasury};

use crate::tests::common::TEST_CREATOR;

#[allow(dead_code)]
pub struct BaseContracts {
    pub luna_vault: Addr,
    pub bluna: Addr,
    pub memory: Addr,
    pub treasury: Addr,
    pub unbond_handler: Addr,
}

/// Creates the basic contract instances needed to test the dapp.
/// luna vault, bluna token, memory contract, treasury contract
pub fn init_contracts(app: &mut TerraApp) -> BaseContracts {
    let owner = Addr::unchecked(TEST_CREATOR);

    // Instantiate bluna Token Contract
    let cw20_token_contract = Box::new(ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));
    let cw20_token_code_id = app.store_code(cw20_token_contract);

    let cw20_instantiate_msg = cw20_base::msg::InstantiateMsg {
        name: String::from("bluna"),
        symbol: String::from("bluna"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };
    // Instantiate bluna Contract
    let cw20_instance = app
        .instantiate_contract(
            cw20_token_code_id,
            owner.clone(),
            &cw20_instantiate_msg,
            &[],
            "bluna token",
            None,
        )
        .unwrap();

    // Upload Treasury Contract
    let treasury_contract = Box::new(ContractWrapper::new_with_empty(
        ::treasury::contract::execute,
        ::treasury::contract::instantiate,
        ::treasury::contract::query,
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

    // Init Memory Contract
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

    // Upload anchor basset hub Contract
    let basset_hub_contract = Box::new(ContractWrapper::new_with_empty(
        anchor_basset_hub::contract::execute,
        anchor_basset_hub::contract::instantiate,
        anchor_basset_hub::contract::query,
    ));
    let basset_hub_code_id = app.store_code(basset_hub_contract);
    let basset_hub_instantiate_msg = basset::hub::InstantiateMsg {
        epoch_period: 0,
        underlying_coin_denom: "uluna".to_string(),
        unbonding_period: 0,
        peg_recovery_fee: Decimal::permille(5),
        er_threshold: Decimal::one(),
        reward_denom: "uusd".to_string(),
        validator: "validator".to_string(),
    };

    // Init anchor basset hub
    let basset_hub_instance = app
        .instantiate_contract(
            basset_hub_code_id,
            owner.clone(),
            &basset_hub_instantiate_msg,
            &[],
            "basset hub",
            None,
        )
        .unwrap();

    // Upload unbond handler contract
    let unbond_handler_contract = Box::new(ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));
    let unbond_handler_code_id = app.store_code(unbond_handler_contract);
    let unbond_handler_instantiate_msg = UnbondHandlerMsg::InstantiateMsg {
        owner: None,
        memory_contract: memory_instance.to_string(),
        expires_in: None,
    };

    // Init anchor basset hub
    let unbond_handler_instance = app
        .instantiate_contract(
            unbond_handler_code_id,
            owner.clone(),
            &unbond_handler_instantiate_msg,
            &[],
            "unbond handler",
            None,
        )
        .unwrap();

    // Upload luna vault Contract
    let luna_vault_contract = Box::new(
        ContractWrapper::new_with_empty(
            luna_vault::contract::execute,
            luna_vault::contract::instantiate,
            luna_vault::contract::query,
        )
        .with_reply(luna_vault::contract::reply),
    );
    let luna_vault_code_id = app.store_code(luna_vault_contract);
    let luna_vault_instantiate_msg = LunaVaultMsg::InstantiateMsg {
        bluna_address: cw20_instance.to_string(),
        cluna_address: "cluna".to_string(),
        astro_lp_address: "astro_lp_address".to_string(),
        astro_factory_address: "astro_factory_address".to_string(),
        treasury_addr: treasury_instance.to_string(),
        memory_addr: memory_instance.to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: LUNA_DENOM.to_string(),
        },
        token_code_id: 148,
        treasury_fee: Decimal::percent(1),
        flash_loan_fee: Decimal::percent(1),
        commission_fee: Decimal::percent(1),
        vault_lp_token_name: Some("vluna token".to_string()),
        vault_lp_token_symbol: Some("vluna".to_string()),
        unbond_handler_code_id,
    };

    // Init Memory Contract
    let luna_vault_instance = app
        .instantiate_contract(
            luna_vault_code_id,
            owner.clone(),
            &luna_vault_instantiate_msg,
            &[],
            "luna vault",
            None,
        )
        .unwrap();

    BaseContracts {
        luna_vault: luna_vault_instance,
        treasury: treasury_instance,
        memory: memory_instance,
        bluna: cw20_instance,
        unbond_handler: unbond_handler_instance,
    }
}

pub fn mock_app() -> TerraApp {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();
    let custom_querier: TerraMockQuerier =
        TerraMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    AppBuilder::new(api, env.block, bank, storage, custom_querier)
    // let custom_handler = CachingCustomHandler::<CustomMsg, Empty>::new();
    // AppBuilder::new().with_custom(custom_handler).build()
}
