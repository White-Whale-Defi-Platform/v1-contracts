use crate::msg::QueryMsg;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use white_whale::{query::memory::query_assets_from_mem, treasury::msg::ValueQueryMsg};

// pub fn handle_value_query(deps: Deps, query: ValueQueryMsg) -> StdResult<Binary> {
//     query_assets_from_mem(deps, memory_addr, '')
//     if query.asset_info.equal(asset)

// }
