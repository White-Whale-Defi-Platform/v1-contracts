#![cfg(test)]

use crate::contract::{execute, instantiate, query, reply};
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{coins, Addr, BlockInfo, Decimal, Empty, Timestamp, Uint128};
use cw20::{Cw20Coin, Cw20Contract};
use cw_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
use terraswap::asset::AssetInfo;
use war_chest::msg::InstantiateMsg;
use white_whale::test_helpers::tswap_mock::{contract_receiver_mock, MockInstantiateMsg};
use white_whale::ust_vault::msg::InstantiateMsg as VaultInstantiateMsg;
// Custom Vault Instant msg func which takes code ID
pub fn instantiate_msg(token_code_id: u64, war_chest: String) -> VaultInstantiateMsg {
    VaultInstantiateMsg {
        anchor_money_market_address: "test_mm".to_string(),
        aust_address: "test_aust".to_string(),
        profit_check_address: "test_profit_check".to_string(),
        warchest_addr: war_chest,
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: token_code_id,
        warchest_fee: Decimal::percent(10u64),
        flash_loan_fee: Decimal::permille(5u64),
        stable_cap: Uint128::from(100_000_000_00000000u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    }
}

pub fn contract_whale_token() -> Box<dyn Contract<Empty>> {
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
        war_chest::contract::execute,
        war_chest::contract::instantiate,
        war_chest::contract::query,
    );
    Box::new(contract)
}

pub fn contract_stablecoin_vault() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
    Box::new(contract)
}

pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();

    App::new(api, env.block, bank, MockStorage::new())
}

#[test]
// setup all contracts needed, and attempt to simply change the stable_cap AS-THE governance contract
// this test attempts to send some WHALE token to a named address on creation
// the gov_staker address then attempts to stake some tokens by sending a Cw20ExecuteMsg which contains a Cw20HookMsg for the gov contract
// the gov_staker address then attempts to create a poll via the same method. The Poll contains the white_whale::ust_vault::msg::ExecuteMsg::SetStableCap message
// the gov_staker casts a Yes vote
// Time passing is simulated
// Poll is ended and then executed
// Verification is done to ensure the proposed change in the gov vote has been performed
fn stablecoin_vault_fees_are_allocated() {
    // Create the owner account
    let owner = Addr::unchecked("owner");

    // Define a mock_app to be used for storing code and instantiating
    let mut router = mock_app();
    router
        .init_bank_balance(&owner, coins(1000, "uusd"))
        .unwrap();
    // Store the stablecoin vault as a code object
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Store the gov contract as a code object
    let warchest_id = router.store_code(contract_warchest());

    // Set the block height and time, we will later modify this to simulate time passing
    let initial_block = BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(1000),
        chain_id: "terra-cosmwasm-testnet".to_string(),
    };
    router.set_block(initial_block);
    // Lastly, store our terrswap mock which is a slimmed down Terraswap with no real functionality
    let terraswap_id = router.store_code(contract_receiver_mock());

    // Store whale token which is a CW20 and get its code ID
    let whale_token_id = router.store_code(contract_whale_token());

    // Create the Whale token giving owner some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "White Whale".to_string(),
        symbol: "WHALE".to_string(),
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let whale_token_instance = router
        .instantiate_contract(whale_token_id, owner.clone(), &msg, &[], "WHALE", None)
        .unwrap();

    // set up cw20 helpers
    let cash = Cw20Contract(whale_token_instance.clone());

    // get owner balance
    let owner_balance = cash.balance(&router, owner.clone()).unwrap();
    // Verify the funds have been received
    assert_eq!(owner_balance, Uint128::new(5000));

    // Setup Warchest
    let chest_msg = InstantiateMsg {
        admin_addr: owner.to_string(),
        whale_token_addr: whale_token_instance.to_string(),
        spend_limit: Uint128::from(1_000_000u128),
    };

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let tswap_addr = router
        .instantiate_contract(
            terraswap_id,
            owner.clone(),
            &MockInstantiateMsg {},
            &[],
            "TSWAP",
            None,
        )
        .unwrap();

    // Setup the warchest contract
    let warchest_addr = router
        .instantiate_contract(
            warchest_id,
            owner.clone(),
            &chest_msg,
            &[],
            "WARCHEST",
            None,
        )
        .unwrap();

    // First prepare an InstantiateMsg for vault contract with the mock terraswap token_code_id
    let vault_msg = instantiate_msg(terraswap_id, warchest_addr.to_string());

    // Next setup the vault with the gov contract as the 'owner'
    let vault_addr = router
        .instantiate_contract(
            vault_id,
            owner.clone(),
            &vault_msg,
            &[],
            "VAULT",
            Some(owner.to_string()),
        )
        .unwrap();
    // Ensure addresses are not equal to each other
    assert_ne!(warchest_addr, vault_addr);
    assert_ne!(vault_addr, tswap_addr);

    // let msg = ExecuteMsg::ProvideLiquidity{
    //     asset: Asset {
    //         info: AssetInfo::NativeToken{denom: "uusd".to_string()},
    //         amount: Uint128::new(1000)
    //     }
    // };
    // let res = router
    //     .execute_contract(owner.clone(), vault_addr.clone(), &msg, &coins(1000, "uusd"))
    //     .unwrap();

    // println!("{:?}", res.events);
    // let msg = Cw20HookMsg::WithdrawLiquidity {};

    // // Prepare cw20 message with our attempt to withdraw tokens, this should incur a fee
    // let send_msg = Cw20ExecuteMsg::Send {
    //     contract: vault_addr.to_string(),
    //     amount: Uint128::new(1),
    //     msg: to_binary(&msg).unwrap(),
    // };
    // let res = router
    //     .execute_contract(owner.clone(), vault_addr.clone(), &send_msg, &[])
    //     .unwrap();
}

// Need to :
//  Setup vault with specified fee share
// deposit N (maybe 100 tokens)
// withdraw n
// verify the share percent was done.
