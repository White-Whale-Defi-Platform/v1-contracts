use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::querier::query_token_balance;

use crate::contract::CommunityFundResult;
use crate::error::CommunityFundError;
use crate::state::{ADMIN, STATE};

/// Transfer WHALE to specified recipient
pub fn spend_whale(
    deps: Deps,
    info: MessageInfo,
    recipient: String,
    fund_contract_addr: Addr,
    amount: Uint128,
) -> CommunityFundResult {
    ADMIN.assert_admin(deps, &info.sender)?;
    check_fund_balance(deps, fund_contract_addr, amount)?;
    let state = STATE.load(deps.storage)?;
    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.whale_token_addr.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer { recipient, amount })?,
        })),
    )
}

/// Call burn on WHALE cw20 token
pub fn burn_whale(
    deps: Deps,
    info: MessageInfo,
    fund_contract_addr: Addr,
    amount: Uint128,
) -> CommunityFundResult {
    ADMIN.assert_admin(deps, &info.sender)?;
    check_fund_balance(deps, fund_contract_addr, amount)?;
    let state = STATE.load(deps.storage)?;
    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.whale_token_addr.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        })),
    )
}

/// Sets new admin
pub fn set_admin(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
) -> Result<Response, CommunityFundError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

/// Checks whether or not the community fund has enough balance to make a transaction
fn check_fund_balance(
    deps: Deps,
    fund_contract_addr: Addr,
    amount: Uint128,
) -> Result<(), CommunityFundError> {
    let state = STATE.load(deps.storage)?;

    let fund_whale_balance =
        query_token_balance(&deps.querier, state.whale_token_addr, fund_contract_addr)?;
    if amount > fund_whale_balance {
        return Err(CommunityFundError::InsufficientFunds(
            amount,
            fund_whale_balance,
        ));
    };

    Ok(())
}
