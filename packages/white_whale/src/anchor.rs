use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg
};
use schemars::JsonSchema;
use std::fmt;
use crate::msg::AnchorMsg;


pub fn try_deposit_to_anchor<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
    anchor_money_market_address: String,
    amount: Coin
) -> StdResult<Response<T>> {
    if amount.denom != "uusd" {
        return Err(StdError::generic_err("Wrong currency. Only UST (denom: uusd) is supported."));
    }

    let msg = CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: anchor_money_market_address,
        msg: to_binary(&AnchorMsg::DepositStable{})?,
        funds: vec![amount]
    });

    Ok(Response::new().add_message(msg))
}


pub fn try_deposit_to_anchor_as_submsg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
    anchor_money_market_address: String,
    amount: Coin,
    id: u64
) -> StdResult<Response<T>> {
    if amount.denom != "uusd" {
        return Err(StdError::generic_err("Wrong currency. Only UST (denom: uusd) is supported."));
    }

    let msg = CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: anchor_money_market_address,
        msg: to_binary(&AnchorMsg::DepositStable{})?,
        funds: vec![amount]
    });

    Ok(Response::new().add_submessage(SubMsg{
        msg,
        gas_limit: None,
        id,
        reply_on: ReplyOn::Success,

    }))
}