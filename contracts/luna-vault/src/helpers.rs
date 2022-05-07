use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, Env, Event, Reply, StdError,
    StdResult, Storage, SubMsgExecutionResponse, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use schemars::JsonSchema;
use std::fmt;

use white_whale::denom::LUNA_DENOM;
use white_whale::fee::Fee;
use white_whale::luna_vault::luna_unbond_handler::msg::Cw20HookMsg::Unbond as UnbondHandlerUnbondMsg;
use white_whale::luna_vault::luna_unbond_handler::msg::ExecuteMsg;
use white_whale::luna_vault::luna_unbond_handler::msg::ExecuteMsg::WithdrawUnbonded as UnbondHandlerWithdrawMsg;
use white_whale::query::terraswap::query_asset_balance;
use white_whale::tax::compute_tax;

use crate::contract::VaultResult;
use crate::error::LunaVaultError;
use crate::pool_info::PoolInfoRaw;
use crate::state::{FEE, STATE};

/// compute total vault value of deposits in LUNA and return a tuple with those values.
/// (total, luna, astro lp, bluna, cluna)
pub fn compute_total_value(
    _env: &Env,
    deps: Deps,
    info: &PoolInfoRaw,
) -> VaultResult<(Uint128, Uint128, Uint128, Uint128, Uint128)> {
    let state = STATE.load(deps.storage)?;
    // get liquid Luna in the vault
    let luna_info = info.asset_infos[0].to_normal(deps.api)?;
    let luna_amount = query_asset_balance(deps, &luna_info, info.contract_addr.clone())?;

    // get Luna from the passive strategy
    // first, get the amount of LP tokens that we have
    // then, query the pool to find out the underlying Luna assets we are entitled to
    let astro_lp_info = info.asset_infos[1].to_normal(deps.api)?;
    let astro_lp_amount = query_asset_balance(deps, &astro_lp_info, info.contract_addr.clone())?;
    let astro_lp_assets: [astroport::asset::Asset; 2] = deps.querier.query_wasm_smart(
        state.astro_lp_address.clone(),
        &astroport::pair_stable_bluna::QueryMsg::Share {
            amount: astro_lp_amount,
        },
    )?;
    // NOTICE: we are assuming that the assets in the LP are equivalent to 1 Luna
    let astroport_lp_value_in_luna = astro_lp_assets
        .iter()
        .fold(Uint128::zero(), |accum, asset| accum + asset.amount);

    // NOTICE: we are assuming that bLuna is equivalent to 1 Luna
    let bluna_value_in_luna = astro_lp_assets
        .iter()
        .find(|asset| asset.info == astroport::asset::token_asset_info(state.bluna_address.clone()))
        .ok_or_else(|| LunaVaultError::generic_err("Failed to get bLuna asset from astro LP"))?
        .amount;
    let cluna_value_in_luna = Uint128::zero();

    let total_deposits_in_luna =
        luna_amount + astroport_lp_value_in_luna + bluna_value_in_luna + cluna_value_in_luna;
    Ok((
        total_deposits_in_luna,
        luna_amount,
        astroport_lp_value_in_luna,
        bluna_value_in_luna,
        cluna_value_in_luna,
    ))
}

pub fn get_withdraw_fee(deps: Deps, amount: Uint128) -> VaultResult<Uint128> {
    let treasury_fee = get_treasury_fee(deps, amount)?;
    //TODO fee from Passive Strategy, i.e. Astroport LP?
    let astroport_lp_fee = Uint128::zero();
    let luna_transfer_fee = compute_tax(
        deps,
        &Coin::new(
            (amount - treasury_fee - astroport_lp_fee).u128(),
            String::from(LUNA_DENOM),
        ),
    )?;
    // Two transfers (passive_strategy (astroport lp) -> vault -> user) so ~2x tax.
    Ok(treasury_fee + astroport_lp_fee + luna_transfer_fee)
}

pub fn get_treasury_fee(deps: Deps, amount: Uint128) -> VaultResult<Uint128> {
    let fee_config = FEE.load(deps.storage)?;
    let fee = fee_config.treasury_fee.compute(amount);
    Ok(fee)
}

/// Checks that the given [Fee] is valid, i.e. it's lower than 100%
pub fn check_fee(fee: Fee) -> VaultResult<Fee> {
    if fee.share >= Decimal::percent(100) {
        return Err(LunaVaultError::InvalidFee {});
    }
    Ok(fee)
}

pub fn get_lp_token_address(deps: &Deps, pool_address: Addr) -> VaultResult<Addr> {
    let pool_info: astroport::asset::PairInfo = deps.querier.query_wasm_smart(
        pool_address,
        &astroport::pair_stable_bluna::QueryMsg::Pair {},
    )?;

    Ok(pool_info.liquidity_token)
}

/// Determine if an event contains a specific key-value pair
pub fn event_contains_attr(event: &Event, key: &str, value: &str) -> bool {
    event
        .attributes
        .iter()
        .any(|attr| attr.key == key && attr.value == value)
}

/// Gets the string value of an attribute from the event
pub fn get_attribute_value_from_event(event: &Event, key: &str) -> Result<String, StdError> {
    Ok(event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == key)
        .ok_or_else(|| StdError::generic_err(format!("cannot find {} attribute", key)))?
        .value)
}

/// Extracts response from reply
pub fn unwrap_reply(reply: Reply) -> StdResult<SubMsgExecutionResponse> {
    reply.result.into_result().map_err(StdError::generic_err)
}

/// Extracts data from reply response
pub fn unwrap_data(response: SubMsgExecutionResponse) -> Result<Binary, StdError> {
    response
        .data
        .ok_or_else(|| StdError::generic_err("Can't get data from reply response"))
}

/// Builds message to send bluna to a handler triggering the unbond action
pub fn update_unbond_handler_state_msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
    unbond_handler: Addr,
    owner: Option<String>,
    expiration_time: Option<u64>,
) -> StdResult<CosmosMsg<T>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: unbond_handler.to_string(),
        msg: to_binary(&ExecuteMsg::UpdateState {
            owner,
            expiration_time,
            memory_contract: None,
        })?,
        funds: vec![],
    }))
}

/// Builds message to send bluna to a handler triggering the unbond action
pub fn unbond_bluna_with_handler_msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
    storage: &dyn Storage,
    bluna_amount: Uint128,
    unbond_handler: &Addr,
) -> StdResult<CosmosMsg<T>> {
    let state = STATE.load(storage)?;
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.bluna_address.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: unbond_handler.to_string(),
            amount: bluna_amount,
            msg: to_binary(&UnbondHandlerUnbondMsg {})?,
        })?,
        funds: vec![],
    }))
}

/// Builds message to withdraw luna from a handler triggering the withdraw_unbonded action
pub fn withdraw_luna_from_handler_msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
    unbond_handler: Addr,
) -> StdResult<CosmosMsg<T>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: unbond_handler.to_string(),
        msg: to_binary(&UnbondHandlerWithdrawMsg {})?,
        funds: vec![],
    }))
}
pub trait ConversionAsset {
    fn to_astroport(self, deps: &Deps) -> Result<astroport::asset::AssetInfo, StdError>;
}

impl ConversionAsset for terraswap::asset::AssetInfo {
    fn to_astroport(self, deps: &Deps) -> Result<astroport::asset::AssetInfo, StdError> {
        Ok(match self {
            terraswap::asset::AssetInfo::NativeToken { denom } => {
                astroport::asset::AssetInfo::NativeToken { denom }
            }
            terraswap::asset::AssetInfo::Token { contract_addr } => {
                astroport::asset::AssetInfo::Token {
                    contract_addr: deps.api.addr_validate(&contract_addr)?,
                }
            }
        })
    }
}
