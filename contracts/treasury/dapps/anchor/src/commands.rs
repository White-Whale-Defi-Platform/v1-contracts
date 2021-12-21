use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal, Deps, Env, Fraction, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::Asset;
use terraswap::pair::{Cw20HookMsg, PoolResponse};

use white_whale::query::terraswap::{query_asset_balance, query_pool};
use white_whale::treasury::dapp_base::common::PAIR_POSTFIX;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::state::{load_contract_addr, STATE};
use white_whale::treasury::msg::send_to_treasury;

use crate::contract::AnchorResult;
use crate::error::AnchorError;
use crate::state::get_asset_info;
use crate::terraswap_msg::{asset_into_swap_msg, deposit_lp_msg};
use crate::utils::has_sufficient_balance;

// Add the custom dapp-specific message commands here


/// Constructs and forwards the anchor deposit_stable message for the treasury
/// The scenario covered here is such that there is UST in the treasury (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the treasury for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_deposit_stable(
    deps: Deps,
    env: Env,
    info: MessageInfo,
) -> AnchorResult {
    // Prepare a deposit_msg using the provided info. 
    // The anchor dapp will then use this message and pass it to the treasury for execution
    let deposit_msg = anchor_deposit_msg(
        deps.as_ref(),
        deps.api.addr_humanize(&state.anchor_money_market_address)?,
        anchor_deposit,
    )?;
}

/// Constructs and forwards the anchor redeem_stable message for the treasury
/// The scenario covered here is such that there is aUST in the treasury (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the treasury for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_redeem_stable(
    deps: Deps,
    env: Env,
    info: MessageInfo,
) -> AnchorResult {
    // Prepare a deposit_msg using the provided info. 
    // The anchor dapp will then use this message and pass it to the treasury for execution
    let deposit_msg = anchor_deposit_msg(
        deps.as_ref(),
        deps.api.addr_humanize(&state.anchor_money_market_address)?,
        anchor_deposit,
    )?;
}