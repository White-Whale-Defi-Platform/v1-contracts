use crate::state::{State, STATE};
use cosmwasm_std::{Deps, Env, StdResult};
use white_whale::memory::queries::query_contract_from_mem;
use white_whale::memory::ANCHOR_BLUNA_HUB_ID;
use white_whale::query::anchor::{UnbondRequestsResponse, WithdrawableUnbondedResponse};

/// Gets the state of the contract
pub(crate) fn query_state(deps: Deps) -> StdResult<State> {
    STATE.load(deps.storage)
}

/// Gets the state of the contract
pub(crate) fn query_withdrawable_unbonded(
    deps: Deps,
    env: Env,
) -> StdResult<WithdrawableUnbondedResponse> {
    let state = STATE.load(deps.storage)?;
    let bluna_hub_address =
        query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;

    // query how much withdrawable_unbonded is on anchor for the given unbond handler
    Ok(white_whale::query::anchor::query_withdrawable_unbonded(
        deps,
        bluna_hub_address,
        env.contract.address,
    )?)
}

/// Gets the state of the contract
pub(crate) fn query_unbond_requests(deps: Deps, env: Env) -> StdResult<UnbondRequestsResponse> {
    let state = STATE.load(deps.storage)?;
    let bluna_hub_address =
        query_contract_from_mem(deps.as_ref(), &state.memory_contract, ANCHOR_BLUNA_HUB_ID)?;

    // query unbond requests on anchor for the given unbond handler
    Ok(white_whale::query::anchor::query_unbond_requests(
        deps,
        bluna_hub_address,
        env.contract.address,
    )?)
}
