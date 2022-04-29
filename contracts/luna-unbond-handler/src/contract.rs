#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use crate::{commands, queries, UnbondHandlerResult};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:luna-unbond-handler";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> UnbondHandlerResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut state = State {
        owner: None,
        expiration_time: None,
        memory_contract: deps.api.addr_validate(&msg.memory_contract)?,
    };

    if let Some(owner) = msg.owner {
        state.owner = Some(deps.api.addr_validate(&owner)?);
        state.expiration_time = Some(env.block.time.seconds());
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", state.owner.unwrap_or(Addr::unchecked("")))
        .add_attribute(
            "expiration_time",
            state.expiration_time.unwrap_or(0).to_string(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> UnbondHandlerResult {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::WithdrawUnbonded {} => commands::withdraw_unbonded_bluna(deps, env),
        ExecuteMsg::SetAdmin { admin } => commands::set_admin(deps, info, admin),
        ExecuteMsg::UpdateState {
            owner,
            expiration_time,
            memory_contract,
        } => commands::update_state(deps, info, owner, expiration_time, memory_contract),
        ExecuteMsg::Callback(msg) => commands::_handle_callback(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
    }
}
