#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StakerInfoResponse,
    StateResponse,
};
use crate::state::{
    read_config, read_staker_info, read_state, store_config, store_state, Config, StakerInfo, State,
};

use cw20::{Cw20ReceiveMsg};

// version info for migration info
// const CONTRACT_NAME: &str = "crates.io:whale-lp-staking";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    store_config(
        deps.storage,
        &Config {
            whale_token: deps.api.addr_canonicalize(&msg.whale_token)?,
            staking_token: deps.api.addr_canonicalize(&msg.staking_token)?,
            distribution_schedule: msg.distribution_schedule,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            last_distributed: _env.block.height,
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, _env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_height } => to_binary(&query_state(deps, block_height)?),
        QueryMsg::StakerInfo {
            staker,
            block_height,
        } => to_binary(&query_staker_info(deps, staker, block_height)?),
    }
}

/// handler function invoked when the governance contract receives
/// a transaction. This is akin to a payable function in Solidity
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            // only staking token contract can execute this message
            if config.staking_token != deps.api.addr_canonicalize(info.sender.as_str())? {
                return Err(StdError::generic_err("unauthorized"));
            }

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            bond(deps, env, cw20_sender, cw20_msg.amount)
        }
        Err(_) => Err(StdError::generic_err("data should be given")),
    }
}

// compute distributed rewards and update global reward index
fn compute_reward(config: &Config, state: &mut State, block_height: u64) {
    if state.total_bond_amount.is_zero() {
        state.last_distributed = block_height;
        return;
    }

    let mut distributed_amount: Uint128 = Uint128::zero();
    for s in config.distribution_schedule.iter() {
        if s.0 > block_height || s.1 < state.last_distributed {
            continue;
        }

        // min(s.1, block_height) - max(s.0, last_distributed)
        let passed_blocks =
            std::cmp::min(s.1, block_height) - std::cmp::max(s.0, state.last_distributed);

        let num_blocks = s.1 - s.0;
        let distribution_amount_per_block: Decimal = Decimal::from_ratio(s.2, num_blocks);
        distributed_amount += distribution_amount_per_block * Uint128::from(passed_blocks as u128);
    }

    state.last_distributed = block_height;
    state.global_reward_index = state.global_reward_index
        + Decimal::from_ratio(distributed_amount, state.total_bond_amount);
}

// withdraw reward to pending reward
fn compute_staker_reward(state: &State, staker_info: &mut StakerInfo) -> StdResult<()> {
    let pending_reward = (staker_info.bond_amount * state.global_reward_index)
        .checked_sub(staker_info.bond_amount * staker_info.reward_index)?;

    staker_info.reward_index = state.global_reward_index;
    staker_info.pending_reward += pending_reward;
    Ok(())
}

/// bond is the handler function allowing a user to send tokens to the staking contract in an attempt to bond them
pub fn bond(_deps: DepsMut, _env: Env, _sender_addr: Addr, _amount: Uint128) -> StdResult<Response> {
    // let sender_addr_raw: CanonicalAddr = deps.api.addr_canonicalize(sender_addr.as_str())?;

    // let config: Config = read_config(deps.storage)?;
    // let mut state: State = read_state(deps.storage)?;
    // let mut staker_info: StakerInfo = read_staker_info(deps.storage, &sender_addr_raw)?;

    // // Compute global reward & staker reward
    // compute_reward(&config, &mut state, env.block.height);
    // compute_staker_reward(&state, &mut staker_info)?;

    // // Increase bond_amount
    // increase_bond_amount(&mut state, &mut staker_info, amount);

    // // Store updated state with staker's staker_info
    // store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    // store_state(deps.storage, &state)?;

    // Ok(Response::new().add_attributes(vec![
    //     ("action", "bond"),
    //     ("owner", sender_addr.as_str()),
    //     ("amount", amount.to_string().as_str()),
    // ]))
    Ok(Response::new())
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        whale_token: deps.api.addr_humanize(&state.whale_token)?.to_string(),
        staking_token: deps.api.addr_humanize(&state.staking_token)?.to_string(),
        distribution_schedule: state.distribution_schedule,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps, block_height: Option<u64>) -> StdResult<StateResponse> {
    let mut state: State = read_state(deps.storage)?;
    if let Some(block_height) = block_height {
        let config = read_config(deps.storage)?;
        compute_reward(&config, &mut state, block_height);
    }

    Ok(StateResponse {
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index: state.global_reward_index,
    })
}

pub fn query_staker_info(
    deps: Deps,
    staker: String,
    block_height: Option<u64>,
) -> StdResult<StakerInfoResponse> {
    let staker_raw = deps.api.addr_canonicalize(&staker)?;

    let mut staker_info: StakerInfo = read_staker_info(deps.storage, &staker_raw)?;
    if let Some(block_height) = block_height {
        let config = read_config(deps.storage)?;
        let mut state = read_state(deps.storage)?;

        compute_reward(&config, &mut state, block_height);
        compute_staker_reward(&state, &mut staker_info)?;
    }

    Ok(StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}
