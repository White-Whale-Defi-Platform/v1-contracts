use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use white_whale::treasury::vault_assets::VaultAsset;

use crate::commands;
use crate::error::HidingGameError;
use crate::state::{Config, CONFIG, ADMIN};
use white_whale::hiding_game::*;
use white_whale::memory::item::Memory;
pub type HidingGameResult = Result<Response, HidingGameError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> HidingGameResult {
    let config = Config {
        dex_arb_addr: deps.api.addr_validate(&msg.dex_arb_addr)?,
        seignorage_addr: deps.api.addr_validate(&msg.seignorage_addr)?,
        vault_addr: deps.api.addr_validate(&msg.vault_addr)?,
        whale: msg.whale,
        rebait_ratio: msg.rebait_ratio,
        memory: Memory {
            address: deps.api.addr_validate(&msg.memory)?,
        },
    };

    // Store the initial config
    CONFIG.save(deps.storage, &config)?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HidingGameResult {
    match msg {
    ExecuteMsg::UpdateConfig {  } => todo!(),
    ExecuteMsg::WhaleTrade { pair, offer, amount, max_spread, belief_price } => todo!(),
    ExecuteMsg::SetAdmin { admin } => todo!(),
}
}


fn _handle_callback(deps: DepsMut, env: Env, info: MessageInfo, msg: CallbackMsg) -> HidingGameResult {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(HidingGameError::NotCallback {});
    }
    match msg {
        CallbackMsg::AfterTrade {} => commands::after_trade(deps, env, info, loan_fee),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        
    }
}

