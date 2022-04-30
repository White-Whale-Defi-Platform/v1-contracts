use crate::state::{State, STATE};
use cosmwasm_std::{Deps, StdResult};

/// Gets the state of the contract
pub(crate) fn query_state(deps: Deps) -> StdResult<State> {
    STATE.load(deps.storage)
}
