use std::fmt;

use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrismMsg {
    Unbond {},
    WithdrawUnbonded {},
}

pub fn prism_cluna_unbond_msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
    cluna_address: Addr,
    cluna_hub_address: Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg<T>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cluna_address.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: cluna_hub_address.to_string(),
            amount,
            msg: to_binary(&PrismMsg::Unbond {})?,
        })?,
        funds: vec![],
    }))
}

pub fn prism_withdraw_unbonded_msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
    cluna_hub_address: Addr,
) -> StdResult<CosmosMsg<T>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cluna_hub_address.to_string(),
        msg: to_binary(&PrismMsg::WithdrawUnbonded {})?,
        funds: vec![],
    }))
}
