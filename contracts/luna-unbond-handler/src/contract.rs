#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

use white_whale::luna_vault::luna_unbond_handler::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use white_whale::luna_vault::luna_unbond_handler::{EXPIRATION_TIME_KEY, OWNER_KEY};

use crate::serde_option::serde_option;
use crate::state::{State, ADMIN, STATE};
use crate::{commands, queries, UnbondHandlerError, UnbondHandlerResult};

// version info for migration info
pub(crate) const CONTRACT_NAME: &str = "crates.io:luna-unbond-handler";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
    }

    if let Some(expires_in) = msg.expires_in {
        let expiration_time = env
            .block
            .time
            .seconds()
            .checked_add(expires_in)
            .ok_or(UnbondHandlerError::WrongExpirationTime {})?;
        state.expiration_time = Some(expiration_time);
    }

    STATE.save(deps.storage, &state)?;

    // set the admin
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute(OWNER_KEY, serde_option(state.owner))
        .add_attribute(EXPIRATION_TIME_KEY, serde_option(state.expiration_time)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> UnbondHandlerResult {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::WithdrawUnbonded {} => commands::withdraw_unbonded_bluna(deps, env, info),
        ExecuteMsg::SetAdmin { admin } => commands::set_admin(deps, info, admin),
        ExecuteMsg::UpdateState {
            owner,
            expiration_time,
            memory_contract,
        } => commands::update_state(deps, info, owner, expiration_time, memory_contract),
        ExecuteMsg::Callback(msg) => commands::handle_callback(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::WithdrawableUnbonded {} => {
            to_binary(&queries::query_withdrawable_unbonded(deps, env)?)
        }
        QueryMsg::UnbondRequests {} => to_binary(&queries::query_unbond_requests(deps, env)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> UnbondHandlerResult {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}
