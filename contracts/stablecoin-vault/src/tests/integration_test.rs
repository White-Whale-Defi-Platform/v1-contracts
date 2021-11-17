#![cfg(test)]

use cosmwasm_std::{to_binary, coins, Addr, BlockInfo, Empty, Uint128, Decimal, Timestamp, QueryRequest, BankQuery};
use cosmwasm_std::testing::{ mock_env, MockApi, MockStorage, MockQuerier, MOCK_CONTRACT_ADDR};
use terra_multi_test::{App, Contract, BankKeeper, ContractWrapper, Executor};
use terra_mocks::TerraMockQuerier;
use crate::contract::{execute, instantiate, query, reply};
use white_whale::ust_vault::msg::{ExecuteMsg};
use terraswap::pair::Cw20HookMsg;
use terraswap::asset::{Asset, AssetInfo};
use white_whale::ust_vault::msg::InstantiateMsg as VaultInstantiateMsg;
use war_chest::msg::{InstantiateMsg};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use white_whale::test_helpers::tswap_mock::{contract_receiver_mock, MockInstantiateMsg, set_liq_token_addr, get_liq_token_addr};
use white_whale::test_helpers::anchor_mock::{contract_anchor_mock, MockInstantiateMsg as AnchorMsg};
use crate::contract::{DEFAULT_LP_TOKEN_NAME, DEFAULT_LP_TOKEN_SYMBOL};
use terra_cosmwasm::TerraQuerier;

// Custom Vault Instant msg func which takes code ID 
// TODO: Clean up func sig or remove
pub fn instantiate_msg(token_code_id: u64, war_chest: String, profit_check_addr: String, anchor_addr:String, aust_address: String) -> VaultInstantiateMsg {
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
        war_chest::contract::execute,
        war_chest::contract::instantiate,
        war_chest::contract::query,
    );
    Box::new(contract)
}

pub fn contract_stablecoin_vault() -> Box<dyn Contract<Empty>>{
    let contract = ContractWrapper::new_with_empty(
        execute,
        instantiate,
        query,
    ).with_reply(reply);
    Box::new(contract)
}

pub fn contract_profit_check() -> Box<dyn Contract<Empty>>{
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


#[test]
// setup all the contracts needed for the Vault
// Set the relevant vault for profit check contract 
// Provide some liquidity with ProvideLiquidity
// Remove some liqudiity with WithdrawLiquidity
// Verify fees have been sent (not working)
fn stablecoin_vault_fees_are_allocated() {
    
    // Create the owner account
    let owner = Addr::unchecked("owner");
    
    // Define a mock_app to be used for storing code and instantiating 
    let mut router = mock_app();
    router.init_bank_balance(&owner, coins(1000, "uusd")).unwrap();
    // Store the stablecoin vault as a code object 
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Store the gov contract as a code object 
    let warchest_id = router.store_code(contract_warchest());
    // Store the profit check needed for the vault on provide and withdrawal of liquidity as well as trading actions 
    let profit_check_id = router.store_code(contract_profit_check());
    let anchor_id = router.store_code(contract_anchor_mock());

    // Set the block height and time, we will later modify this to simulate time passing
    let initial_block = BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(1000),
        chain_id: "terra-cosmwasm-testnet".to_string()
    };
    router.set_block(initial_block);
    // Lastly, store our terrswap mock which is a slimmed down Terraswap with no real functionality
    let terraswap_id = router.store_code(contract_receiver_mock());

    

    // Store whale token which is a CW20 and get its code ID
    let cw20_code_id = router.store_code(contract_cw20_token());

    // Create the Whale token giving owner some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "White Whale".to_string(),
        symbol: "WHALE".to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let whale_token_instance = router
        .instantiate_contract(cw20_code_id, owner.clone(), &msg, &[], "WHALE", None)
        .unwrap();

    // Create the Whale token giving owner some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Anchor UST".to_string(),
        symbol: "aUST".to_string(),
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let aust_token_instance = router
        .instantiate_contract(cw20_code_id, owner.clone(), &msg, &[], "aUST", None)
        .unwrap();

    
    // set up cw20 helpers
    let cash = Cw20Contract(whale_token_instance.clone());

    // get owner balance
    let owner_balance = cash.balance(&router, owner.clone()).unwrap();
    // Verify the funds have been received 
    assert_eq!(owner_balance, Uint128::new(5000));

    // Setup Warchest
    let chest_msg = InstantiateMsg{
        admin_addr: owner.to_string(),
        whale_token_addr: whale_token_instance.to_string(),
        spend_limit: Uint128::from(1_000_000u128),
    };

    // Setup Profit Check
    let chest_msg = InstantiateMsg{
        admin_addr: owner.to_string(),
        whale_token_addr: whale_token_instance.to_string(),
        spend_limit: Uint128::from(1_000_000u128),
    };

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let tswap_addr = router
        .instantiate_contract(terraswap_id, owner.clone(), &MockInstantiateMsg{}, &[], "TSWAP", None)
        .unwrap();

    let profit_check_msg = white_whale::profit_check::msg::InstantiateMsg {
        vault_address: tswap_addr.to_string(),
        denom: "uusd".to_string(),
    };

    

    // Setup the warchest contract
    let warchest_addr = router
        .instantiate_contract(warchest_id, owner.clone(), &chest_msg, &[], "WARCHEST", None)
        .unwrap();

    // Setup the profit check contract
    let profit_check_addr = router
        .instantiate_contract(profit_check_id, owner.clone(), &profit_check_msg, &[], "PROFIT", None)
        .unwrap();

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let anchor_addr = router
        .instantiate_contract(anchor_id, owner.clone(), &AnchorMsg{}, &[], "TSWAP", None)
        .unwrap();
        
    // First prepare an InstantiateMsg for vault contract with the mock terraswap token_code_id
    let vault_msg = instantiate_msg(terraswap_id,warchest_addr.to_string(), profit_check_addr.to_string(), anchor_addr.to_string(), aust_token_instance.to_string());
    
    // Next setup the vault with the gov contract as the 'owner'
    let vault_addr = router
        .instantiate_contract(vault_id, owner.clone(), &vault_msg, &[], "VAULT", Some(owner.to_string()))
        .unwrap();


    // Make a mock LP token
    let msg = cw20_base::msg::InstantiateMsg {
        name: DEFAULT_LP_TOKEN_NAME.to_string(),
        symbol: DEFAULT_LP_TOKEN_SYMBOL.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let lp_token = router
        .instantiate_contract(cw20_code_id, vault_addr.clone(), &msg, &[], DEFAULT_LP_TOKEN_SYMBOL, Some(vault_addr.to_string()))
        .unwrap();
    // Set the address of the liquidity token mock
    set_liq_token_addr(Addr::unchecked("Contract #7").to_string());

    // Need to give a mocked token to user
    // Prepare 
    let send_msg = Cw20ExecuteMsg::Transfer {
        recipient: lp_token.to_string(),
        amount: Uint128::new(1000),
    };
    let res = router
        .execute_contract(owner.clone(), lp_token.clone(), &send_msg, &[])
        .unwrap();


    // Ensure addresses are not equal to each other
    assert_ne!(warchest_addr, vault_addr);
    assert_ne!(vault_addr, tswap_addr);


    // Hook up the vault and profit check
    let msg = white_whale::profit_check::msg::ExecuteMsg::SetVault{
        vault_address: vault_addr.to_string()
    };
    let _ = router
        .execute_contract(owner.clone(), profit_check_addr.clone(), &msg, &[])
        .unwrap();

    // Provide some liqudity in UST
    let msg = ExecuteMsg::ProvideLiquidity{
        asset: Asset {
            info: AssetInfo::NativeToken{denom: "uusd".to_string()},
            amount: Uint128::new(1000)
        }
    };
    let res = router
        .execute_contract(owner.clone(), vault_addr.clone(), &msg, &coins(1000, "uusd"))
        .unwrap();
    
    println!("{:?}", res.events);
    set_liq_token_addr(lp_token.to_string());

    // Withdraw some liquidity 
    let msg = Cw20HookMsg::WithdrawLiquidity {};
    let withdraw_amount = Uint128::new(100);
    // Prepare cw20 message with our attempt to withdraw tokens, this should incur a fee
    let send_msg = Cw20ExecuteMsg::Send {
        contract: vault_addr.to_string(),
        amount: withdraw_amount,
        msg: to_binary(&msg).unwrap(),
    };
    let res = router
        .execute_contract(owner.clone(), Addr::unchecked("Contract #7"), &send_msg, &[])
        .unwrap();
    println!("{:?}", res.events);
    
    // let resp = router.wrap().query_all_balances(warchest_addr.clone());
    // println!("{:?}", resp);
    // assert_ne!(resp.unwrap().amount, Uint128::zero())
    // set up a helper for UST
    let lp = Cw20Contract(Addr::unchecked("Contract #7").clone());

    // Verify warchest has received some fees (WIP)
    // ensure our balances
    let war_chest_bal = lp.balance(&router, warchest_addr.clone()).unwrap();
    assert_eq!(war_chest_bal, withdraw_amount.checked_div(Uint128::new(10)).unwrap());

}


// Need to :
//  Setup vault with specified fee share
// deposit N (maybe 100 tokens)
// withdraw n
// verify the share percent was done.