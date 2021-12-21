use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use schemars::_serde_json::to_value;
use white_whale::query::memory::{query_assets_from_mem, query_contracts_from_mem};

use crate::msg::{AssetQueryResponse, ContractQueryResponse};

pub fn query_assets(deps: Deps, env: Env, asset_names: Vec<String>) -> StdResult<Binary> {
    let assets = query_assets_from_mem(deps, env.contract.address, asset_names)?;
    to_binary(&AssetQueryResponse { assets: to_value(assets).unwrap()})
}

pub fn query_contract(deps: Deps, env: Env, names: Vec<String>) -> StdResult<Binary> {
    let contracts = query_contracts_from_mem(deps, env.contract.address, names)?;
    to_binary(&ContractQueryResponse { contracts })
}
