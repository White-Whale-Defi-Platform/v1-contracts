use cosmwasm_std::{to_binary, Binary, Deps, StdResult, Env};

use white_whale::query::memory::query_assets_from_mem;

use crate::msg::AssetQueryResponse;

pub fn query_assets(deps: Deps, env: Env, asset_names: Vec<String>) -> StdResult<Binary> {
    
    let assets = query_assets_from_mem(deps, env.contract.address, asset_names)?;
    to_binary(&AssetQueryResponse {
        assets
    })
}
