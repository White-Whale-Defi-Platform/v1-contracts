use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Deps, StdResult};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::AssetInfo;

use super::queries::{query_contracts_from_mem, query_assets_from_mem};

// Struct that holds address
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Memory {
    pub address: Addr,
}

impl Memory {
    // Raw Query to Memory contract
    pub fn query_contracts(
        &self,
        deps: Deps,
        contract_names: &[String],
    ) -> StdResult<BTreeMap<String, Addr>> {
        query_contracts_from_mem(deps, &self.address, contract_names)
    }
    
    // Raw Query to Memory contract
    pub fn query_assets(
        &self,
        deps: Deps,
        asset_names: &[String],
    ) -> StdResult<BTreeMap<String, AssetInfo>> {
        query_assets_from_mem(deps, &self.address, asset_names)
    }
}