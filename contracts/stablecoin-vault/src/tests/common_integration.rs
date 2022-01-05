use crate::contract::{execute, instantiate, query, reply};
use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Decimal, Empty, Uint128};
use terra_mocks::TerraMockQuerier;
use terra_multi_test::{App, BankKeeper, Contract, ContractWrapper};
use terraswap::asset::AssetInfo;
use white_whale::ust_vault::msg::InstantiateMsg as VaultInstantiateMsg;

// Custom Vault Instant msg func which takes code ID
// TODO: Clean up func sig or remove
pub fn instantiate_msg(
    token_code_id: u64,
    war_chest: String,
    profit_check_addr: String,
    anchor_addr: String,
    aust_address: String,
) -> VaultInstantiateMsg {
    VaultInstantiateMsg {
        anchor_money_market_address: anchor_addr,
        aust_address: aust_address,
        profit_check_address: profit_check_addr,
        warchest_addr: war_chest,
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: token_code_id,
        warchest_fee: Decimal::percent(10u64),
        flash_loan_fee: Decimal::permille(5u64),
        commission_fee: Decimal::permille(8u64),
        stable_cap: Uint128::from(100_000_000_000_000u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    }
}

pub fn contract_cw20_token() -> Box<dyn Contract<Empty>> {
    // Instantiate WHALE Token Contract
    let whale_token_contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(whale_token_contract)
}

pub fn contract_warchest() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        treasury::contract::execute,
        treasury::contract::instantiate,
        treasury::contract::query,
    );
    Box::new(contract)
}

pub fn contract_stablecoin_vault() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
    Box::new(contract)
}

pub fn contract_profit_check() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        profit_check::contract::execute,
        profit_check::contract::instantiate,
        profit_check::contract::query,
    );
    Box::new(contract)
}

pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let custom_querier: TerraMockQuerier =
        TerraMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    App::new(api, env.block, bank, MockStorage::new(), custom_querier)
    // let custom_handler = CachingCustomHandler::<CustomMsg, Empty>::new();
    // AppBuilder::new().with_custom(custom_handler).build()
}
