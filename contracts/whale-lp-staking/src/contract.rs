#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, read_config, store_config, store_state, read_state};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:whale-lp-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    store_config(
        &mut deps.storage,
        &Config {
            whale_token: deps.api.canonical_address(&msg.whale_token)?,
            staking_token: deps.api.canonical_address(&msg.staking_token)?,
            distribution_schedule: msg.distribution_schedule,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            last_distributed: env.block.height,
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        },
    )?;

    Ok(InitResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
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

/// bond is the handler function allowing a user to send tokens to the staking contract in an attempt to bond them
pub fn bond(deps: DepsMut, env: Env, sender_addr: Addr, amount: Uint128) -> StdResult<Response> {
    let sender_addr_raw: CanonicalAddr = deps.api.addr_canonicalize(sender_addr.as_str())?;

    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;
    let mut staker_info: StakerInfo = read_staker_info(deps.storage, &sender_addr_raw)?;

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.height);
    compute_staker_reward(&state, &mut staker_info)?;

    // Increase bond_amount
    increase_bond_amount(&mut state, &mut staker_info, amount);

    // Store updated state with staker's staker_info
    store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "bond"),
        ("owner", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

}
