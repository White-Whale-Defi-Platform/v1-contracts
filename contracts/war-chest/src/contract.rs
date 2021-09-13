#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};
// use cw2::set_contract_version;
// use cw_controllers::Admin;

use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{read_config, store_config, Config, ADMIN};

// version info for migration info
// const CONTRACT_NAME: &str = "crates.io:war-chest";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    store_config(
        deps.storage,
        &Config {
            gov_contract: deps.api.addr_canonicalize(&msg.gov_contract)?,
            whale_token: deps.api.addr_canonicalize(&msg.whale_token)?,
            spend_limit: msg.spend_limit,
        },
    )?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

// Routers; here is a separate router which handles Execution of functions on the contract or performs a contract Query
// Each router function defines a number of handlers using Rust's pattern matching to
// designated how each ExecutionMsg or QueryMsg will be handled.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Spend { recipient, amount } => spend(deps, info, recipient, amount),
        ExecuteMsg::UpdateConfig { spend_limit } => update_config(deps, info, spend_limit),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        gov_contract: deps.api.addr_humanize(&state.gov_contract)?.to_string(),
        whale_token: deps.api.addr_humanize(&state.whale_token)?.to_string(),
        spend_limit: state.spend_limit,
    };

    Ok(resp)
}

// ExecutionMsg handlers

/// Spend
/// Owner can execute spend operation to send
/// `amount` of WHALE token to a `recipient` which could be another contract
pub fn spend(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    // ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    if config.gov_contract != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    if config.spend_limit < amount {
        return Err(ContractError::TooMuchSpend {});
    }

    let whale_token = deps.api.addr_humanize(&config.whale_token)?.to_string();
    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: whale_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "spend"),
            ("recipient", recipient.as_str()),
            ("amount", &amount.to_string()),
        ]))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    spend_limit: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if config.gov_contract != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(spend_limit) = spend_limit {
        config.spend_limit = spend_limit;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}
