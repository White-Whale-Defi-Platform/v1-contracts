#![cfg(test)]

use cosmwasm_std::{coins, Addr, Empty, Response, Uint128, Decimal};
use cosmwasm_std::testing::{ mock_env, MockApi, MockStorage};
use cw_multi_test::{App, Contract, BankKeeper, ContractWrapper, Executor};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::tests::common::{
    mock_env_height, DEFAULT_EXPIRATION_PERIOD, DEFAULT_PROPOSAL_DEPOSIT, DEFAULT_TIMELOCK_PERIOD,
    DEFAULT_VOTING_PERIOD,DEFAULT_THRESHOLD, DEFAULT_QUORUM, DEFAULT_FIX_PERIOD, TEST_CREATOR, TEST_VOTER, TEST_VOTER_2, TEST_VOTER_3, VOTING_TOKEN,
};
use stablecoin_vault::tests::instantiate::{instantiate_msg as vault_instan_msg};
use crate::tests::instantiate::{instantiate_msg as gov_instan_msg};

use stablecoin_vault::contract::{execute, instantiate, query, reply};
use stablecoin_vault::response::{MsgInstantiateContractResponse};
use stablecoin_vault::error::StableVaultError;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{InstantiateMsg, instantiate, query};

use schemars::JsonSchema;
use std::fmt::Debug;
use terra_cosmwasm::{create_swap_msg, TerraMsgWrapper};
 
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
    );
    Box::new(contract)
}

pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
  
    App::new(api, env.block, bank, MockStorage::new())
  }

#[test]
// receive cw20 tokens and release upon approval
fn goverance_can_update_the_slippage_parameter_of_vault() {
    // set personal balance
    let owner = Addr::unchecked("owner");
    let init_funds = coins(2000, "btc");

    let mut router = mock_app();

    // Store the stablecoin vault as a code object 
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Next prepare an instantiate message for that contract
    let vault_msg = vault_instan_msg();
    // Store the gov contract as a code object 
    let gov_id = router.store_code(contract_gov());
    let gov_msg = gov_instan_msg();

    // First setup the gov 
    let gov_addr = router
        .instantiate_contract(gov_id, owner.clone(), &gov_msg, &[], "GOV", None)
        .unwrap();

    
    // Next setup the vault 
    let vault_addr = router
        .instantiate_contract(vault_id, gov_addr.clone(), &vault_msg, &[], "VAULT", None)
        .unwrap();
    // they are different
    assert_ne!(gov_addr, vault_addr);
    // TODO: Improve such that a Poll is created with the Gov contract and the Poll contains a message to 
    // change the slippage param on the vault
    // This would be the proper way to update it as it is not expected
    // let slippage_change_msg = stablecoin_vault::msg::ExecuteMsg::SetSlippage {
    //     slippage: Decimal::percent(5u64)
    // };

    // let res = router
    //     .execute_contract(gov_addr.clone(), vault_addr.clone(), &slippage_change_msg, &[])
    //     .unwrap();
    // println!("{:?}", res.events);
}


