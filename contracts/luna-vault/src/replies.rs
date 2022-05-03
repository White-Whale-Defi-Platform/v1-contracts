use crate::contract::VaultResult;
use crate::helpers::{event_contains_attr, get_attribute_value_from_event};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{POOL_INFO, UNBOND_HANDLERS_ASSIGNED, UNBOND_HANDLER_EXPIRATION_TIMES};
use cosmwasm_std::{attr, Api, DepsMut, Response, StdError};
use white_whale::luna_vault::luna_unbond_handler::{EXPIRATION_TIME_KEY, OWNER_KEY};

/// Executes after the token contract instantiation occurs successfully
/// Stores the liquidity token address
pub fn after_token_instantiation(
    deps: &DepsMut,
    response: &MsgInstantiateContractResponse,
) -> VaultResult<Response> {
    let liquidity_token = deps.api.addr_validate(response.get_contract_address())?;

    POOL_INFO.update(deps.storage, |mut meta| -> VaultResult<_> {
        meta.liquidity_token = liquidity_token;
        Ok(meta)
    })?;

    return Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token.to_string()));
}

/// Executes after the unbond contract instantiation occurs successfully
/// Stores the new unbond handler information into the respective state items
pub fn after_unbond_handler_instantiation(
    deps: &DepsMut,
    response: &MsgInstantiateContractResponse,
) -> VaultResult<Response> {
    let unbond_handler_contract = deps.api.addr_validate(response.get_contract_address())?;
    let events = msg.result.unwrap().events;

    let event = events
        .iter()
        .find(|event| event_contains_attr(event, "action", "instantiate"))
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    // get owner from event
    let owner_string = get_attribute_value_from_event(event, OWNER_KEY);
    let owner = deps.api.addr_validate(&owner_string)?;
    UNBOND_HANDLERS_ASSIGNED.save(deps.storage, &owner, &&unbond_handler_contract)?;

    // get expiration_time from event
    let expiration_time_string = get_attribute_value_from_event(event, EXPIRATION_TIME_KEY);
    let expiration_time = expiration_time_string.parse::<u64>()?;
    UNBOND_HANDLER_EXPIRATION_TIMES.save(
        deps.storage,
        &unbond_handler_contract,
        &expiration_time,
    )?;

    return Ok(Response::new().add_attributes(vec![
        attr("action", "unbond_handler_instantiate"),
        attr("owner", owner_string),
        attr(
            "unbond_handler_contract",
            unbond_handler_contract.to_string(),
        ),
        attr("expiration_time", expiration_time_string),
    ]));
}
