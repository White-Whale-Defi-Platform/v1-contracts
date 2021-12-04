use crate::state::get_asset_info;
use cosmwasm_std::{Addr, Deps, Uint128};
use white_whale::query::terraswap::query_asset_balance;
use white_whale::treasury::dapp_base::error::DAppError;

/// Checks if the given address has enough tokens with a given offer_id
pub fn has_sufficient_balance(
    deps: Deps,
    offer_id: &str,
    address: &Addr,
    required: Uint128,
) -> Result<(), DAppError> {
    // Load asset
    let info = get_asset_info(deps, offer_id)?;
    // Get balance and check
    if query_asset_balance(deps, &info, address.clone())? < required {
        return Err(DAppError::Broke {});
    }
    Ok(())
}
