use cosmwasm_std::{Addr, Binary, CosmosMsg, WasmMsg,Decimal, Deps, DepsMut, Env, Fraction, MessageInfo, Response, StdError, StdResult, Uint128, entry_point, to_binary};


use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{Cw20HookMsg,PoolResponse};
use cw20::Cw20ExecuteMsg;

use crate::error::DAppError;
use crate::msg::{CallbackMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, ADDRESS_BOOK, ADMIN, STATE};
use crate::terraswap_msg::*;
use white_whale::convert::convert_to_asset;
use white_whale::query::terraswap::{query_asset_balance, query_pool};

type DAppResult = Result<Response, DAppError>;
const PAIR_POSTFIX: &str = "_pair";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> DAppResult {
    let state = State {
        treasury_address: deps.api.addr_canonicalize(&msg.treasury_address)?,
        trader: deps.api.addr_canonicalize(&msg.trader)?,
    };

    // Store the initial config
    STATE.save(deps.storage, &state)?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DAppResult {
    match msg {
        // TODO: Add functions
        ExecuteMsg::ProvideLiquidity {
            pool_id,
            main_asset_id,
            amount,
        } => provide_liquidity(deps.as_ref(), env, info, main_asset_id, pool_id, amount),
        ExecuteMsg::WithdrawLiquidity { pool_id, amount } => {
            withdraw_liquidity(deps.as_ref(), env, info, pool_id, amount)
        }
        ExecuteMsg::SwapAsset {
            offer_id,
            ask_id,
            amount,
        } => terraswap_swap(deps.as_ref(), env, info, offer_id, ask_id, amount),
        ExecuteMsg::UpdateConfig {
            treasury_address,
            trader,
        } => update_config(deps, info, treasury_address, trader),
        ExecuteMsg::SetAdmin { admin } => {
            ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

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
fn _handle_callback(deps: DepsMut, env: Env, info: MessageInfo, msg: CallbackMsg) -> DAppResult {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(DAppError::NotCallback {});
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

pub fn provide_liquidity(
    deps: Deps,
    env: Env,
    msg_info: MessageInfo,
    main_asset_id: String,
    pool_id: String,
    amount: Uint128,
) -> DAppResult {

    let state = STATE.load(deps.storage)?;
    // Check if caller is trader.
    if msg_info.sender != deps.api.addr_humanize(&state.trader)? {
        return Err(DAppError::Unauthorized {});
    }

    let treasury_address = deps.api.addr_humanize(&state.treasury_address)?;
    // Get assets from address_book
    let base_asset = convert_to_asset(
        deps,
        ADDRESS_BOOK.load(deps.storage, main_asset_id.as_str())?,
    )?;

    let base_asset_balance = query_asset_balance(deps, &base_asset, treasury_address)?;
    let pair_address = deps
        .api
        .addr_validate(ADDRESS_BOOK.load(deps.storage, pool_id.as_str())?.as_str())?;

    if base_asset_balance > amount {
        return Err(DAppError::Broke {});
    }

    let pool_info: PoolResponse = query_pool(deps, &pair_address)?;
    let asset_1 = &pool_info.assets[0];
    let asset_2 = &pool_info.assets[1];

    let ratio = Decimal::from_ratio(asset_1.amount, asset_2.amount);

    let mut second_asset: Asset;

    if asset_2.info == base_asset {
        second_asset = *asset_1;
        second_asset.amount = ratio * amount;
    } else {
        second_asset = *asset_2;
        second_asset.amount = ratio.inv().unwrap_or_default() * amount;
    }

    let second_asset_balance = query_asset_balance(deps, &second_asset.info, treasury_address)?;

    if second_asset_balance < second_asset.amount {
        return Err(DAppError::Broke {});
    }
    let msgs: Vec<CosmosMsg> = deposit_lp_msg(
        deps,
        [
        Asset {
            info: base_asset,
            amount,
        },second_asset],
        pair_address,
    )?;

    // Deposit lp msg either returns a bank send msg or it returns a
    // increase allowance msg that will be called by the contract. 
    Ok(Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_address.to_string(),
                msg: to_binary(&)?,
                funds: vec![],
            }))
        )
}

pub fn withdraw_liquidity(
    deps: Deps,
    env: Env,
    msg_info: MessageInfo,
    lp_token_id: String,
    amount: Uint128,
) -> DAppResult {
    let state = STATE.load(deps.storage)?;
    let treasury_address = deps.api.addr_humanize(&state.treasury_address)?;

    // get assets from address_book
    let lp_token_address = Addr::unchecked(ADDRESS_BOOK.load(deps.storage, lp_token_id.as_str())?); 
    let pair_address = Addr::unchecked(ADDRESS_BOOK.load(deps.storage, (lp_token_id+PAIR_POSTFIX).as_str())?); 
    let lp_asset = convert_to_asset(deps, lp_token_address.into_string())?;
    let lp_balance = query_asset_balance(deps, &lp_asset, treasury_address)?;

    if lp_balance < amount {
        return Err(DAppError::Broke {});
    }

    let withdraw_msg: Binary = to_binary(&Cw20HookMsg::WithdrawLiquidity{})?;

    let cw20_msg = Cw20ExecuteMsg::Send{
        contract: lp_token_address.into_string(),
        amount,
        msg: withdraw_msg,
    };
    Ok(Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pair_address.to_string(),
        msg: to_binary(&cw20_msg)?,
        funds: vec![],
    })))
}



//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

// After the arb this function returns the funds to the vault.
fn after_successful_action_callback(deps: DepsMut, env: Env) -> DAppResult {
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
) -> DAppResult {
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
) -> DAppResult {
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

// https://users.rust-lang.org/t/updating-object-fields-given-dynamic-json/39049/2
