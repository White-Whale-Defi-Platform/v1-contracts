use cosmwasm_std::{Addr, Coin, CosmosMsg, Deps, Env, Fraction, MessageInfo, Response, StdError, Uint128};
use terraswap::asset::AssetInfo;

use white_whale::anchor::{anchor_bluna_unbond_msg, anchor_deposit_msg, anchor_withdraw_msg, anchor_withdraw_unbonded_msg};
use white_whale::denom::UST_DENOM;
use white_whale::query::anchor::query_aust_exchange_rate;
use white_whale::treasury::dapp_base::common::{ANCHOR_BLUNA_HUB_ID, ANCHOR_MONEY_MARKET_ID, AUST_TOKEN_ID, BLUNA_TOKEN_ID};
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::state::BASESTATE;
use white_whale::treasury::msg::send_to_treasury;

use crate::contract::AnchorResult;

// Add the custom dapp-specific message commands here

/// Constructs and forwards the anchor deposit_stable message for the treasury
/// The scenario covered here is such that there is UST in the treasury (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the treasury for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_deposit_stable(
    deps: Deps,
    _env: Env,
    msg_info: MessageInfo,
    ust_deposit_amount: Uint128,
) -> AnchorResult {
    let state = BASESTATE.load(deps.storage)?;
    // Check if caller is trader.
    if msg_info.sender != state.trader {
        return Err(BaseDAppError::Unauthorized {});
    }

    let treasury_address = &state.treasury_address;

    // Get anchor money market address
    let anchor_address = state
        .memory
        .query_contract(deps, &String::from(ANCHOR_MONEY_MARKET_ID))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    // Prepare a deposit_msg using the provided info.
    // The anchor dapp will then use this message and pass it to the treasury for execution
    let deposit_msg: CosmosMsg = anchor_deposit_msg(
        deps,
        anchor_address,
        Coin::new(ust_deposit_amount.u128(), UST_DENOM),
    )?;
    messages.push(deposit_msg);
    Ok(Response::new().add_message(send_to_treasury(messages, treasury_address)?))
}

/// Constructs and forwards the anchor redeem_stable message for the treasury
/// The scenario covered here is such that there is aUST in the treasury (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the treasury for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_redeem_stable(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    ust_to_withdraw: Uint128,
) -> AnchorResult {
    let state = BASESTATE.load(deps.storage)?;
    // Check if caller is trader.
    if info.sender != state.trader {
        return Err(BaseDAppError::Unauthorized {});
    }

    let treasury_address = &state.treasury_address;

    // Get anchor money market address
    let anchor_address = state
        .memory
        .query_contract(deps, &String::from(ANCHOR_MONEY_MARKET_ID))?;

    // Get aUST address
    let aust_address = state
        .memory
        .query_contract(deps, &String::from(AUST_TOKEN_ID))?;

    let mut messages: Vec<CosmosMsg> = vec![];

    let aust_exchange_rate = query_aust_exchange_rate(env, deps, anchor_address.to_string())?;

    // Prepare a deposit_msg using the provided info.
    // The anchor dapp will then use this message and pass it to the treasury for execution
    let withdraw_msg = anchor_withdraw_msg(
        aust_address,
        anchor_address,
        ust_to_withdraw * aust_exchange_rate.inv().unwrap(),
    )?;
    messages.push(withdraw_msg);
    Ok(Response::new().add_message(send_to_treasury(messages, treasury_address)?))
}


/// Constructs and forwards the anchor unbond message for the treasury
/// The scenario covered here is such that there is bLuna in the treasury (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the treasury for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_unbond(
    deps: Deps,
    _env: Env,
    msg_info: MessageInfo,
    bluna_amount: Uint128,
) -> AnchorResult {
    let state = BASESTATE.load(deps.storage)?;
    // Check if caller is trader.
    if msg_info.sender != state.trader {
        return Err(BaseDAppError::Unauthorized {});
    }

    let treasury_address = &state.treasury_address;

    // Get anchor bluna hub address
    let bluna_hub_address = state
        .memory
        .query_contract(deps, &String::from(ANCHOR_BLUNA_HUB_ID))?;

    // Get anchor bluna token address
    let bluna_address: Addr = match state
        .memory
        .query_asset(deps, &String::from(BLUNA_TOKEN_ID))? {
        AssetInfo::Token { contract_addr } => Addr::unchecked(contract_addr),
        _ => return Err(BaseDAppError::Std(StdError::generic_err("bLuna token asset identified as Native in memory.")))
    };

    let mut messages: Vec<CosmosMsg> = vec![];
    // Prepare the unbond msg using the provided info.
    // The anchor dapp will then use this message and pass it to the treasury for execution
    let unbond_msg: CosmosMsg = anchor_bluna_unbond_msg(
        bluna_address,
        bluna_hub_address,
        bluna_amount,
    )?;
    messages.push(unbond_msg);
    Ok(Response::new().add_message(send_to_treasury(messages, treasury_address)?))
}

/// Constructs and forwards the anchor withdraw unbonded message for the treasury
/// The scenario covered here is such that there is unbonded Luna ready to be withdrawn for the treasury (or whatever similar framework you attach this dapp too)
/// and the anchor-dapp acts as an envoy preparing and providing the message to the treasury for execution
/// Caller address -> anchor-dapp -> Treasury executes message prepared by the anchor-dapp invoked by the caller address which is an admin
pub fn handle_withdraw_unbonded(
    deps: Deps,
    _env: Env,
    msg_info: MessageInfo,
) -> AnchorResult {
    let state = BASESTATE.load(deps.storage)?;
    // Check if caller is trader.
    if msg_info.sender != state.trader {
        return Err(BaseDAppError::Unauthorized {});
    }

    let treasury_address = &state.treasury_address;

    // Get anchor bluna hub address
    let bluna_hub_address = state
        .memory
        .query_contract(deps, &String::from(ANCHOR_BLUNA_HUB_ID))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    // Prepare the withdraw unbonded msg using the provided info.
    // The anchor dapp will then use this message and pass it to the treasury for execution
    let withdraw_unbonded_msg: CosmosMsg = anchor_withdraw_unbonded_msg(bluna_hub_address)?;
    messages.push(withdraw_unbonded_msg);
    Ok(Response::new().add_message(send_to_treasury(messages, treasury_address)?))
}
