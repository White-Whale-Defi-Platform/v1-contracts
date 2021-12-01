use cosmwasm_std::{Deps, StdResult};
use crate::treasury::dapp_base::msg::StateResponse;
use crate::treasury::dapp_base::state::{ADDRESS_BOOK, STATE};

pub fn try_query_config(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(StateResponse {
        treasury_address: deps
            .api
            .addr_humanize(&state.treasury_address)?
            .into_string(),
        trader: deps.api.addr_humanize(&state.trader)?.into_string(),
    })
}

pub fn try_query_addressbook(deps: Deps, id: String) -> StdResult<String> {
    ADDRESS_BOOK.load(deps.storage, id.as_str())
}
