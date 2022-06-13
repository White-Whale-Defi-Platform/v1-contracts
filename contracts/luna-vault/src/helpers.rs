use std::fmt;

use astroport::asset::Asset;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, Env, Event, Reply, StdError,
    StdResult, Storage, SubMsgExecutionResponse, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use white_whale::denom::LUNA_DENOM;
use white_whale::fee::Fee;
use white_whale::luna_vault::luna_unbond_handler::msg::Cw20HookMsg::Unbond as UnbondHandlerUnbondMsg;
use white_whale::luna_vault::luna_unbond_handler::msg::ExecuteMsg;
use white_whale::luna_vault::luna_unbond_handler::msg::ExecuteMsg::WithdrawUnbonded as UnbondHandlerWithdrawMsg;
use white_whale::memory::queries::query_contract_from_mem;
use white_whale::memory::{ANCHOR_BLUNA_HUB_ID, PRISM_CLUNA_HUB_ID};
use white_whale::query::terraswap::query_asset_balance;
use white_whale::query::{anchor, prism};
use white_whale::tax::compute_tax;

use crate::contract::VaultResult;
use crate::error::LunaVaultError;
use crate::pool_info::PoolInfoRaw;
use crate::state::{FEE, STATE};

/// Represents the total value in the vault
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalValue {
    pub total_value_in_luna: Uint128,
    pub luna_amount: Uint128,
    pub astroport_lp_value_in_luna: Uint128,
    pub bluna_value_in_luna: Uint128,
    pub cluna_value_in_luna: Uint128,
    pub bluna_value_burning_in_luna: Uint128,
    pub cluna_value_burning_in_luna: Uint128,
}

/// compute total vault value of deposits in LUNA and return a tuple with those values.
/// (total, luna, astro lp, bluna, cluna)
pub fn compute_total_value(_env: &Env, deps: Deps, info: &PoolInfoRaw) -> VaultResult<TotalValue> {
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

    // NOTICE: we are assuming that cLuna is equivalent to 1 Luna
    let cluna_info = info.asset_infos[3].to_normal(deps.api)?;
    let cluna_value_in_luna = query_asset_balance(deps, &cluna_info, info.contract_addr.clone())?;

    // amount of bluna burning on Anchor
    let bluna_hub_address =
        query_contract_from_mem(deps, &state.memory_address, ANCHOR_BLUNA_HUB_ID)?;
    let bluna_value_burning_in_luna =
        anchor::query_unbond_requests(deps, bluna_hub_address, info.contract_addr.clone())?
            .requests
            .iter()
            .fold(Uint128::zero(), |acc, unbond_request| {
                acc + unbond_request.1 // pending unbond amount
            });

    // amount of cluna burning on Prism
    let cluna_hub_address =
        query_contract_from_mem(deps, &state.memory_address, PRISM_CLUNA_HUB_ID)?;
    let cluna_value_burning_in_luna =
        prism::query_unbond_requests(deps, cluna_hub_address, info.contract_addr.clone())?
            .requests
            .iter()
            .fold(Uint128::zero(), |acc, unbond_request| {
                acc + unbond_request.1 // pending unbond amount
            });

    let total_deposits_in_luna = luna_amount
        + astroport_lp_value_in_luna
        + bluna_value_in_luna
        + cluna_value_in_luna
        + bluna_value_burning_in_luna
        + cluna_value_burning_in_luna;
    Ok(TotalValue {
        total_value_in_luna: total_deposits_in_luna,
        luna_amount,
        astroport_lp_value_in_luna,
        bluna_value_in_luna,
        cluna_value_in_luna,
        bluna_value_burning_in_luna,
        cluna_value_burning_in_luna,
    })
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
    println!("Making query");

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
    triggered_by: Addr,
) -> StdResult<CosmosMsg<T>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: unbond_handler.to_string(),
        msg: to_binary(&UnbondHandlerWithdrawMsg {
            triggered_by_addr: triggered_by.to_string(),
        })?,
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

/// Gets the amount of shares to withdraw from a given `lp_address` so that the `lp0 + lp1` = `total_amount`.
#[allow(dead_code)]
pub fn get_split_share_amount(
    deps: &Deps,
    lp_address: Addr,
    _total_amount: Uint128,
) -> VaultResult<Uint128> {
    let _pool_info: astroport::pair::PoolResponse = deps
        .querier
        .query_wasm_smart(lp_address, &astroport::pair_stable_bluna::QueryMsg::Pool {})?;

    Ok(Uint128::zero())
}

/// Gets the amount of shares to withdraw from a given `lp_address` to get the `requested_info` amount.
pub fn get_share_amount(
    deps: &Deps,
    lp_address: Addr,
    requested_asset: Asset,
) -> VaultResult<Uint128> {
    // the amount we will get from the LP pool in the form of the desired requested_token is equal to (x*z)/y = b
    // where x = the share amount we withdraw, y = the share total, z = the pool size
    // we can therefore get the desired share amount by rearranging to the form
    // x = (b*y)/z
    let pool_info: astroport::pair::PoolResponse = deps
        .querier
        .query_wasm_smart(lp_address, &astroport::pair_stable_bluna::QueryMsg::Pool {})?;

    let pool_desired_asset = pool_info
        .assets
        .iter()
        .find(|asset| asset.info == requested_asset.info)
        .ok_or(LunaVaultError::NoSwapAvailable {})?;

    // before we do x = (b * y) / z, we must account for integer division rounding down
    // the resultant share that we withdraw must be rounded up from the calculation, or we will get 1 uluna too little.
    // to get the correct amount (i.e., do ceiling division), we calculate the remainder of the operation and make the numerator
    // add the difference between the denominator and remainder
    // example: pool sizes of 35323332730080000, 2889842192163. If we did not do ceiling division, we would get 12223.2739 = 12223
    // share to withdraw. However, this will only give us 9999 uluna, not 10000uluna. Therefore, we need to do
    // (35323332730080000 + (2889842192163 - (35323332730080000 % 2889842192163))) / 2889842192163 which gives a perfect 12224 share amount
    // which is the correct amount.
    let numerator = requested_asset.amount.checked_mul(pool_info.total_share)?;
    let denominator = pool_desired_asset.amount;

    let remainder = numerator.checked_rem(denominator)?;

    // add to numerator so that division is rounding up instead of down
    let numerator = numerator.checked_add(denominator.checked_sub(remainder)?)?;

    let share_to_withdraw = numerator.checked_div(denominator)?;

    Ok(share_to_withdraw)
}
