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
// use terraswap::pair::{InstantiateMsg, instantiate, query};

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



