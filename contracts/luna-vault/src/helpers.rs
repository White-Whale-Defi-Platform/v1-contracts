use cosmwasm_std::{Coin, Decimal, Deps, Env, StdResult, Uint128};
use terraswap::asset::AssetInfo;
use terraswap::querier::{query_balance};
use white_whale::denom::LUNA_DENOM;
use white_whale::fee::Fee;

use white_whale::query::astroport::query_astro_lp_exchange_rate;
use white_whale::tax::compute_tax;
use crate::error::LunaVaultError;

use crate::pool_info::PoolInfoRaw;
use crate::state::{FEE, STATE};

/// compute total vault value of deposits in LUNA and return a tuple with those values.
/// (total, luna, astro lp, bluna, cluna)
pub fn compute_total_value(
    _env: &Env,
    deps: Deps,
    info: &PoolInfoRaw,
) -> StdResult<(Uint128, Uint128, Uint128, Uint128, Uint128)> {
    let _state = STATE.load(deps.storage)?;
    let luna_info = info.asset_infos[0].to_normal(deps.api)?;
    let luna_denom = match luna_info {
        AssetInfo::Token { .. } => String::default(),
        AssetInfo::NativeToken { denom } => denom,
    };
    let luna_amount = query_balance(&deps.querier, info.contract_addr.clone(), luna_denom)?;

    //TODO fix query_astro_lp_exchange_rate
    let astro_lp_info = info.asset_infos[1].to_normal(deps.api)?;
    let astro_lp_amount = astro_lp_info.query_pool(&deps.querier, deps.api, info.contract_addr.clone())?;
    let astro_lp_exchange_rate = query_astro_lp_exchange_rate()?;

    let astroport_lp_value_in_luna = astro_lp_amount * astro_lp_exchange_rate;
    //TODO fix bluna and cluna values in luna
    let bluna_value_in_luna = Uint128::zero();
    let cluna_value_in_luna = Uint128::zero();

    let total_deposits_in_luna = luna_amount + astroport_lp_value_in_luna + bluna_value_in_luna + cluna_value_in_luna;
    Ok((total_deposits_in_luna, luna_amount, astroport_lp_value_in_luna, bluna_value_in_luna, cluna_value_in_luna))
}

pub fn get_withdraw_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
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

pub fn get_treasury_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let fee_config = FEE.load(deps.storage)?;
    let fee = fee_config.treasury_fee.compute(amount);
    Ok(fee)
}

/// Checks that the given [Fee] is valid, i.e. it's lower than 100%
pub fn check_fee(fee: Fee) -> Result<Fee, LunaVaultError> {
    if fee.share >= Decimal::percent(100) {
        return Err(LunaVaultError::InvalidFee {});
    }
    Ok(fee)
}
