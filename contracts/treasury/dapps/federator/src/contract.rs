use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::error::FederatorError;
use crate::msg::{CallbackMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, INSTRUCTION_SET, ADMIN, STATE};

type FederatorResult = Result<Response, FederatorError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> FederatorResult {
    let state = State {
        treasury_address: deps.api.addr_validate(&msg.treasury_address)?,
        trader: deps.api.addr_validate(&msg.trader)?,
    };

    // Store the initial config
    STATE.save(deps.storage, &state)?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> FederatorResult {
    match msg {
        // TODO: Add functions
        ExecuteMsg::UpdateConfig {
            treasury_address,
            trader,
        } => update_config(deps, info, treasury_address, trader),
        ExecuteMsg::SetAdmin { admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
            Ok(Response::default()
                .add_attribute("previous admin", previous_admin)
                .add_attribute("admin", admin))
        }
        ExecuteMsg::UpdateAddressBook { to_add, to_remove } => {
            update_address_book(deps, info, to_add, to_remove)
        }
        ExecuteMsg::Callback(msg) => _handle_callback(deps, env, info, msg),
    }
}

//----------------------------------------------------------------------------------------
//  PRIVATE FUNCTIONS
//----------------------------------------------------------------------------------------

// TODO: Callback to be implemented
fn _handle_callback(deps: DepsMut, env: Env, info: MessageInfo, msg: CallbackMsg) -> FederatorResult {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(FederatorError::NotCallback {});
    }
    match msg {
        CallbackMsg::AfterSuccessfulActionCallback {} => {
            after_successful_action_callback(deps, env)
        } // Possibility to add more callbacks.
    }
}
//----------------------------------------------------------------------------------------
//  EXECUTE FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

// TODO: implement

//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

// After the arb this function returns the funds to the vault.
fn after_successful_action_callback(deps: DepsMut, env: Env) -> FederatorResult {
    // Fill
    Ok(Response::new())
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

pub fn update_address_book(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(String, String)>,
    to_remove: Vec<String>,
) -> FederatorResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (name, new_address) in to_add.into_iter() {
        // update function for new or existing keys
        let insert = |vault_asset: Option<String>| -> StdResult<String> {
            match vault_asset {
                Some(_) => Err(StdError::generic_err("Asset already present.")),
                None => Ok(new_address),
            }
        };
        ADDRESS_BOOK.update(deps.storage, name.as_str(), insert)?;
    }

    for name in to_remove {
        ADDRESS_BOOK.remove(deps.storage, name.as_str());
    }

    Ok(Response::new().add_attribute("action", "updated address book"))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    treasury_address: Option<String>,
    trader: Option<String>,
) -> FederatorResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    let api = deps.api;

    if let Some(treasury_address) = treasury_address {
        state.treasury_address = api.addr_canonicalize(&treasury_address)?;
    }

    if let Some(trader) = trader {
        state.trader = api.addr_canonicalize(&trader)?;
    }

    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("Update:", "Successfull"))
}

//----------------------------------------------------------------------------------------
//  QUERY HANDLERS
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&try_query_config(deps)?),
        // Todo: add addressbook query
    }
}

pub fn try_query_config(deps: Deps) -> StdResult<State> {
    let state = STATE.load(deps.storage)?;
    Ok(state)
}
