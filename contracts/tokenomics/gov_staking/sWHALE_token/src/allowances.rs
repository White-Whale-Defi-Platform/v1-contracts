use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, Uint128};
use cw20::Cw20ReceiveMsg;
use cw20_base::allowances::deduct_allowance;
use cw20_base::ContractError;

use crate::core;
use crate::state::CONFIG;

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    let config = CONFIG.load(deps.storage)?;
    let messages = core::transfer(deps.storage, &config, owner_addr, rcpt_addr, amount)?;

    let res = Response::new()
        .add_messages(messages)
        .add_attribute("action", "transfer_from")
        .add_attribute("from", owner)
        .add_attribute("to", recipient)
        .add_attribute("by", info.sender)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&contract)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    let config = CONFIG.load(deps.storage)?;
    let transfer_messages =
        core::transfer(deps.storage, &config, owner_addr, rcpt_addr, amount, true)?;

    let res = Response::new()
        .add_attribute("action", "send_from")
        .add_attribute("from", &owner)
        .add_attribute("to", &contract)
        .add_attribute("by", &info.sender)
        .add_attribute("amount", amount)
        .add_messages(transfer_messages)
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.to_string(),
                amount,
                msg,
            }
            .into_cosmos_msg(contract)?,
        );
    Ok(res)
}
