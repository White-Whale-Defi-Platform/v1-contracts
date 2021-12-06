use cosmwasm_std::{Binary, Deps, StdResult, to_binary};

use crate::treasury::dapp_base::msg::{BaseQueryMsg, BaseStateResponse};
use crate::treasury::dapp_base::state::{ADDRESS_BOOK, STATE};

/// Handles the common base queries
pub fn handle_base_query(deps: Deps, query: BaseQueryMsg) -> StdResult<Binary> {
    match query {
        BaseQueryMsg::Config {} => to_binary(&try_query_config(deps)?),
        BaseQueryMsg::AddressBook { id } => to_binary(&try_query_addressbook(deps, id)?),
    }
}

pub fn try_query_config(deps: Deps) -> StdResult<BaseStateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(BaseStateResponse {
        treasury_address: state.treasury_address
            .into_string(),
        trader: state.trader.into_string(),
    })
}

pub fn try_query_addressbook(deps: Deps, id: String) -> StdResult<String> {
    ADDRESS_BOOK.load(deps.storage, id.as_str())
}
