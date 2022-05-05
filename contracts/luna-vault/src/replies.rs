use cosmwasm_std::{attr, DepsMut, Event, Response, StdError};

use white_whale::luna_vault::luna_unbond_handler::{EXPIRATION_TIME_KEY, OWNER_KEY};

use crate::contract::VaultResult;
use crate::error::LunaVaultError;
use crate::helpers::unbond_bluna_with_handler_msg;
use crate::helpers::{event_contains_attr, get_attribute_value_from_event};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{
    POOL_INFO, UNBOND_CACHE, UNBOND_HANDLERS_ASSIGNED, UNBOND_HANDLER_EXPIRATION_TIMES,
};

/// Executes after the token contract instantiation occurs successfully
/// Stores the liquidity token address
pub fn after_token_instantiation(
    deps: DepsMut,
    response: MsgInstantiateContractResponse,
) -> VaultResult<Response> {
    let liquidity_token = deps.api.addr_validate(response.get_contract_address())?;

    POOL_INFO.update(deps.storage, |mut meta| -> VaultResult<_> {
        meta.liquidity_token = liquidity_token.clone();
        Ok(meta)
    })?;

    Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token.to_string()))
}

/// Executes after the unbond contract instantiation occurs successfully
/// Stores the new unbond handler information into the respective state items
pub fn after_unbond_handler_instantiation(
    deps: DepsMut,
    response: MsgInstantiateContractResponse,
    events: Vec<Event>,
) -> VaultResult<Response> {
    let unbond_handler_contract = deps.api.addr_validate(response.get_contract_address())?;

    let event = events
        .iter()
        .find(|event| event_contains_attr(event, "action", "instantiate"))
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    // get owner from event
    let owner_string = get_attribute_value_from_event(event, OWNER_KEY)?;
    let owner = deps.api.addr_validate(&owner_string)?;
    UNBOND_HANDLERS_ASSIGNED.save(deps.storage, owner.clone(), &unbond_handler_contract)?;

    // get expiration_time from event
    let expiration_time_string = get_attribute_value_from_event(event, EXPIRATION_TIME_KEY)?;
    let expiration_time = expiration_time_string.parse::<u64>()?;
    UNBOND_HANDLER_EXPIRATION_TIMES.save(
        deps.storage,
        unbond_handler_contract.clone(),
        &expiration_time,
    )?;

    // get data from the cache to execute the unbond operation after the unbond handler is created
    let unbond_data_cache_option = UNBOND_CACHE.may_load(deps.storage)?;
    if unbond_data_cache_option.is_none() {
        return Err(LunaVaultError::UnbondHandlerMissingDataCache {});
    }

    let unbond_data_cache =
        unbond_data_cache_option.ok_or(LunaVaultError::UnbondDataCacheError {})?;
    // make sure the cached owner corresponds to the one fetched from the instantiation msg
    let cached_owner = unbond_data_cache.clone().owner;
    if cached_owner != owner {
        return Err(LunaVaultError::UnbondHandlerMismatchingDataCache {});
    }

    // get bluna amount from cache
    let bluna_amount = unbond_data_cache.bluna_amount;

    // send bluna to unbond handler
    let unbond_msg =
        unbond_bluna_with_handler_msg(deps.storage, bluna_amount, &unbond_handler_contract)?;

    // clear the unbond cache
    UNBOND_CACHE.remove(deps.storage);

    Ok(Response::new().add_message(unbond_msg).add_attributes(vec![
        attr("action", "unbond_handler_instantiate"),
        attr("owner", owner_string),
        attr(
            "unbond_handler_contract",
            unbond_handler_contract.to_string(),
        ),
        attr("expiration_time", expiration_time_string),
    ]))
}
