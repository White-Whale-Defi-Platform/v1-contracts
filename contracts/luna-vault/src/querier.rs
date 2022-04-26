use cosmwasm_std::{Coin, Decimal, Deps, Uint128};
use terra_cosmwasm::TerraQuerier;

use crate::contract::VaultResult;

pub fn from_micro(amount: Uint128) -> Decimal {
    Decimal::from_ratio(amount, Uint128::from(1000000u64))
}

pub fn query_market_price(deps: Deps, offer_coin: Coin, ask_denom: String) -> VaultResult<Uint128> {
    let querier = TerraQuerier::new(&deps.querier);
    let response = querier.query_swap(offer_coin, ask_denom)?;
    Ok(response.receive.amount)
}
