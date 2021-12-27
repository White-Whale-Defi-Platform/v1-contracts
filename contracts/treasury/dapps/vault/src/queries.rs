use cosmwasm_std::{Binary, Deps, StdResult, to_binary};
use crate::msg::QueryMsg;
use white_whale::treasury::msg::ValueQueryMsg;
/// Handles the common base queries
pub fn handle_query(deps: Deps, query: QueryMsg) -> StdResult<Binary> {
    match query {
        QueryMsg::State {} => to_binary(&try_query_config(deps)?),
        QueryMsg::ValueQueryMsg::Value { id } => to_binary(&try_query_addressbook(deps, id)?),
    }
}
