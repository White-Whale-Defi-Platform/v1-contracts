#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};

use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, STATE, State};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    STATE.save(deps.storage, &State{
        whale_token_addr: deps.api.addr_canonicalize(&msg.whale_token_addr)?,
        spend_limit: msg.spend_limit,
    })?;
    let admin_addr = Some(deps.api.addr_validate(&msg.admin_addr)?);
    ADMIN.set(deps, admin_addr)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Spend { recipient, amount } => spend(deps, info, recipient, amount),
        ExecuteMsg::UpdateSpendLimit { spend_limit } => update_spend_limit(deps, info, spend_limit),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let resp = ConfigResponse {
        whale_token_addr: deps.api.addr_humanize(&state.whale_token_addr)?.to_string(),
        spend_limit: state.spend_limit,
    };

    Ok(resp)
}

pub fn spend(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let state = STATE.load(deps.storage)?;
    if state.spend_limit < amount {
        return Err(ContractError::TooMuchSpend {});
    }

    let whale_token_addr = deps.api.addr_humanize(&state.whale_token_addr)?.to_string();
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: whale_token_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount,
            })?,
        }))
        .add_attributes(vec![
            ("action", "spend"),
            ("recipient", recipient.as_str()),
            ("amount", &amount.to_string()),
        ]))
}

pub fn update_spend_limit(
    deps: DepsMut,
    info: MessageInfo,
    spend_limit: Uint128,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut state = STATE.load(deps.storage)?;
    let previous_spend_limit = state.spend_limit;
    state.spend_limit = spend_limit;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "update_spend_limit")
        .add_attribute("previous spend limit", previous_spend_limit.to_string())
        .add_attribute("spend limit", spend_limit.to_string()))
}