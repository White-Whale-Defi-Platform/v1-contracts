use cosmwasm_std::{
    Binary, CosmosMsg, Decimal, Deps, Env, Fraction, MessageInfo, Response, to_binary, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::Asset;
use terraswap::pair::{Cw20HookMsg, PoolResponse};

use white_whale::query::terraswap::{query_asset_balance, query_pool};
use white_whale::treasury::dapp_base::common::{DAppResult, PAIR_POSTFIX};
use white_whale::treasury::dapp_base::error::DAppError;
use white_whale::treasury::dapp_base::state::{load_contract_addr, STATE};
use white_whale::treasury::msg::send_to_treasury;

use crate::state::get_asset_info;
use crate::terraswap_msg::{asset_into_swap_msg, deposit_lp_msg};
use crate::utils::has_sufficient_balance;

/// Constructs and forwards the terraswap provide_liquidity message
pub fn provide_liquidity(
    deps: Deps,
    msg_info: MessageInfo,
    main_asset_id: String,
    pool_id: String,
    amount: Uint128,
) -> DAppResult {
    let state = STATE.load(deps.storage)?;
    // Check if caller is trader.
    if msg_info.sender != deps.api.addr_validate(&state.trader.as_str())? {
        return Err(DAppError::Unauthorized {});
    }

    let treasury_address = deps.api.addr_validate(&state.treasury_address.as_str())?;

    // Get lp token address
    let pair_address = load_contract_addr(deps, &pool_id)?;

    // Get pool info
    let pool_info: PoolResponse = query_pool(deps, &pair_address)?;
    let asset_1 = &pool_info.assets[0];
    let asset_2 = &pool_info.assets[1];

    let ratio = Decimal::from_ratio(asset_1.amount, asset_2.amount);

    let main_asset_info = get_asset_info(deps, &main_asset_id)?;
    let main_asset = Asset {
        info: main_asset_info,
        amount,
    };
    let mut first_asset: Asset;
    let mut second_asset: Asset;

    // Determine second asset and required amount to do a 50/50 LP
    if asset_2.info.equal(&main_asset.info) {
        first_asset = asset_1.clone();
        first_asset.amount = ratio * amount;
        second_asset = main_asset;
    } else {
        second_asset = asset_2.clone();
        second_asset.amount = ratio.inv().unwrap_or_default() * amount;
        first_asset = main_asset;
    }

    // Does the treasury have enough of these assets?
    let first_asset_balance =
        query_asset_balance(deps, &first_asset.info, treasury_address.clone())?;
    let second_asset_balance =
        query_asset_balance(deps, &second_asset.info, treasury_address.clone())?;
    if second_asset_balance < second_asset.amount || first_asset_balance < first_asset.amount {
        return Err(DAppError::Broke {});
    }

    let msgs: Vec<CosmosMsg> = deposit_lp_msg(deps, [second_asset, first_asset], pair_address)?;

    // Deposit lp msg either returns a bank send msg or a
    // increase allowance msg for each asset.
    Ok(Response::new().add_message(send_to_treasury(msgs, &treasury_address)?))
}

/// Constructs withdraw liquidity msg and forwards it to treasury
pub fn withdraw_liquidity(
    deps: Deps,
    msg_info: MessageInfo,
    lp_token_id: String,
    amount: Uint128,
) -> DAppResult {
    let state = STATE.load(deps.storage)?;
    if msg_info.sender != deps.api.addr_validate(&state.trader.as_str())? {
        return Err(DAppError::Unauthorized {});
    }
    let treasury_address = deps.api.addr_validate(&state.treasury_address.as_str())?;

    // get lp token address
    let lp_token_address = load_contract_addr(deps, &lp_token_id)?;
    let pair_address = load_contract_addr(deps, &(lp_token_id.clone() + PAIR_POSTFIX))?;

    // Check if the treasury has enough lp tokens
    has_sufficient_balance(deps, &lp_token_id, &treasury_address, amount)?;

    // Msg that gets called on the pair address.
    let withdraw_msg: Binary = to_binary(&Cw20HookMsg::WithdrawLiquidity {})?;

    // cw20 send message that transfers the LP tokens to the pair address
    let cw20_msg = Cw20ExecuteMsg::Send {
        contract: pair_address.into_string(),
        amount,
        msg: withdraw_msg,
    };

    // Call on LP token.
    let lp_call = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_address.into_string(),
        msg: to_binary(&cw20_msg)?,
        funds: vec![],
    });

    Ok(Response::new().add_message(send_to_treasury(vec![lp_call], &treasury_address)?))
}

/// Function constructs terraswap swap messages and forwards them to the treasury
#[allow(clippy::too_many_arguments)]
pub fn terraswap_swap(
    deps: Deps,
    _env: Env,
    msg_info: MessageInfo,
    offer_id: String,
    pool_id: String,
    amount: Uint128,
    max_spread: Option<Decimal>,
    belief_price: Option<Decimal>,
) -> DAppResult {
    let state = STATE.load(deps.storage)?;
    let treasury_address = deps.api.addr_validate(&state.treasury_address.as_str())?;

    // Check if caller is trader
    if msg_info.sender != deps.api.addr_validate(&state.trader.as_str())? {
        return Err(DAppError::Unauthorized {});
    }

    // Check if treasury has enough to swap
    has_sufficient_balance(deps, &offer_id, &treasury_address, amount)?;

    let pair_address = load_contract_addr(deps, &pool_id)?;

    let offer_asset_info = get_asset_info(deps, &offer_id)?;

    let swap_msg = vec![asset_into_swap_msg(
        deps,
        pair_address,
        Asset {
            info: offer_asset_info,
            amount,
        },
        max_spread,
        belief_price,
        None,
    )?];

    Ok(Response::new().add_message(send_to_treasury(swap_msg, &treasury_address)?))
}
