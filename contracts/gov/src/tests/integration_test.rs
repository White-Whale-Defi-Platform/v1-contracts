#![cfg(test)]

use cosmwasm_std::{coins, Addr, BlockInfo, Empty, Response, Uint128, Decimal, Timestamp, to_binary};
use cosmwasm_std::testing::{ mock_env, MockApi, MockStorage};
use cw_multi_test::{App, Contract, BankKeeper, ContractWrapper, Executor};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::tests::common::{
    mock_env_height, DEFAULT_EXPIRATION_PERIOD, DEFAULT_PROPOSAL_DEPOSIT, DEFAULT_TIMELOCK_PERIOD,
    DEFAULT_VOTING_PERIOD,DEFAULT_THRESHOLD, DEFAULT_QUORUM, DEFAULT_FIX_PERIOD, TEST_CREATOR, TEST_VOTER, TEST_VOTER_2, TEST_VOTER_3, VOTING_TOKEN,
};
use crate::state::{
    bank_read, poll_voter_read, state_read, Cw20HookMsg, OrderBy, PollExecuteMsg, PollResponse,
    PollStatus, PollsResponse, StakerResponse, State, VoteOption, VoterInfo, VotersResponse,
};
use crate::tests::instantiate::{instantiate_msg as gov_instan_msg};

use stablecoin_vault::contract::{execute, instantiate, query, reply};
use stablecoin_vault::response::{MsgInstantiateContractResponse};
use stablecoin_vault::error::StableVaultError;
use terraswap::asset::{Asset, AssetInfo};
use stablecoin_vault::pool_info::{PoolInfo};
use schemars::JsonSchema;
use std::fmt::Debug;
use terra_cosmwasm::{create_swap_msg, TerraMsgWrapper};
use crate::tests::tswap_mock::{contract_receiver_mock, MockInstantiateMsg};
use crate::tests::poll::{create_poll_msg};
use white_whale::ust_vault::msg::InstantiateMsg as VaultInstantiateMsg;

use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};

// Custom Vault Instant msg func which takes code ID 
pub fn instantiate_msg(token_code_id: u64) -> VaultInstantiateMsg {
    VaultInstantiateMsg {
        anchor_money_market_address: "test_mm".to_string(),
        aust_address: "test_aust".to_string(),
        profit_check_address: "test_profit_check".to_string(),
        warchest_addr: "warchest".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: token_code_id,
        warchest_fee: Decimal::percent(10u64),
        flash_loan_fee: Decimal::permille(5u64),
        stable_cap: Uint128::from(100_000_000u64),
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

pub fn contract_gov() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
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


pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
  
    App::new(api, env.block, bank, MockStorage::new())
  }

#[test]
// setup all contracts needed, and attempt to simply change the stable_cap AS-THE governance contract
// would be good to test via a pool also but cw-multi-test seems to have different ways of doing most things to unit tests
fn gov_can_update_the_stable_cap_parameter_of_vault() {

    let new_stable_cap_value = 900_000_000u64;
    // set initial contracts owner
    let owner = Addr::unchecked("owner");
    // Define a mock_app to be used for storing code and instantiating 
    let mut router = mock_app();

    // Store the stablecoin vault as a code object 
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Store the gov contract as a code object 
    let gov_id = router.store_code(contract_gov());

    // Set the block height and time, we will later modify this to simulate time passing
    let initial_block = BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(1000),
        chain_id: "terra-cosmwasm-testnet".to_string()
    };
    router.set_block(initial_block);
    // Lastly, store our terrswap mock which is a slimmed down Terraswap with no real functionality
    let terraswap_id = router.store_code(contract_receiver_mock());

    // First prepare an InstantiateMsg for vault contract with the mock terraswap token_code_id
    let vault_msg = instantiate_msg(terraswap_id);
    // Next prepare the Gov contract InstantiateMsg
    let gov_msg = gov_instan_msg();

    let whale_token_id = router.store_code(contract_whale_token());

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

    // Create the gov staker account
    let gov_staker = Addr::unchecked("gov_staker");
    // Next, give the gov_staker some whale to stake with 
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: gov_staker.to_string(),
        amount: Uint128::new(1000),
    };
    let res = router
        .execute_contract(owner.clone(), whale_token_instance.clone(), &msg, &[])
        .unwrap();

    // set up cw20 helpers
    let cash = Cw20Contract(whale_token_instance.clone());

    // get staker balance
    let staker_balance = cash.balance(&router, gov_staker.clone()).unwrap();
    // Verify the funds have been received 
    assert_eq!(staker_balance, Uint128::new(1000));




    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let tswap_addr = router
        .instantiate_contract(terraswap_id, owner.clone(), &MockInstantiateMsg{}, &[], "TSWAP", None)
        .unwrap();

    // Setup the gov contract
    let gov_addr = router
        .instantiate_contract(gov_id, owner.clone(), &gov_msg, &[], "GOV", None)
        .unwrap();

    
    // Next setup the vault with the gov contract as the 'owner'
    let vault_addr = router
        .instantiate_contract(vault_id, gov_addr.clone(), &vault_msg, &[], "VAULT", None)
        .unwrap();
    // Ensure addresses are not equal to each other
    assert_ne!(gov_addr, vault_addr);
    assert_ne!(vault_addr, tswap_addr);
    assert_ne!(gov_addr, tswap_addr);
    
    // Get the current stable_cap to later compare with
    let config_msg = white_whale::ust_vault::msg::VaultQueryMsg::Config{};
    let pool_response:PoolInfo = router.wrap()
        .query_wasm_smart(vault_addr.clone(), &config_msg).unwrap();
    let original_stable_cap: Uint128 = pool_response.stable_cap;

    // TODO: Improve such that a Poll is created with the Gov contract and the Poll contains a message to 
    // change the slippage param on the vault
    // This would be the proper way to update it as it is not expected
    let stable_cap_change_msg = to_binary(&white_whale::ust_vault::msg::ExecuteMsg::SetStableCap {
        stable_cap: Uint128::from(new_stable_cap_value),
    }).unwrap();

    // push two execute msgs to the list
    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 1u64,
            contract: vault_addr.to_string(),
            msg: stable_cap_change_msg,
        }
    ];

    let create_msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs.clone()),
    );

    let res = router
        .execute_contract(gov_addr.clone(), gov_addr.clone(), &create_msg, &[])
        .unwrap();

    println!("{:?}", res.events);

    // Now simulate passing of time 
    // Set the block height and time, we will later modify this to simulate time passing
    let new_block = BlockInfo {
        height: DEFAULT_VOTING_PERIOD,
        time: Timestamp::from_seconds(DEFAULT_VOTING_PERIOD),
        chain_id: "terra-cosmwasm-testnet".to_string()
    };
    router.set_block(new_block);


    // Still TODO:
    // Registering a cw20 as the gov token, needs a mocked cw20 
    // Staking voting tokens for a given address who votes on the poll
    // vote on poll with cast vote
    // end poll
    // execute poll
    // Below config call can confirm if it worked 

    // Get the new stable_cap
    let config_msg = white_whale::ust_vault::msg::VaultQueryMsg::Config{};
    let pool_response:PoolInfo = router.wrap()
        .query_wasm_smart(vault_addr.clone(), &config_msg).unwrap();
    let new_stable_cap: Uint128 = pool_response.stable_cap;
    assert_ne!(original_stable_cap, new_stable_cap);
    assert_eq!(new_stable_cap, Uint128::from(new_stable_cap_value))
}


