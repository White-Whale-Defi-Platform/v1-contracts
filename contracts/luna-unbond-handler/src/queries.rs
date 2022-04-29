use cosmwasm_std::{Deps, StdResult};
use crate::state::{State, STATE};

pub(crate) fn query_state(deps: Deps) -> StdResult<State> {
    Ok(STATE.load(&deps.storage)?)
}
