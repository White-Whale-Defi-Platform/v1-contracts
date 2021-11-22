use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdError, StdResult, Storage, Uint128, WasmMsg};

use cw20_base::state::{BALANCES, TOKEN_INFO};
use cw20_base::ContractError;

use crate::Config;

/// Deduct amount from sender balance and add it to recipient balance
/// Returns messages to be sent on the final response
pub fn transfer(
    storage: &mut dyn Storage,
    config: &Config,
    sender_address: Addr,
    recipient_address: Addr,
    amount: Uint128,
) {
    if sender_address == recipient_address {
        return Err(StdError::generic_err("Sender and recipient cannot be the same").into());
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let sender_previous_balance = decrease_balance(storage, &sender_address, amount)?;
    let recipient_previous_balance = increase_balance(storage, &recipient_address, amount)?;
    let total_supply = TOKEN_INFO.load(storage)?.total_supply;
}

/// Lower user balance and commit to store, returns previous balance
pub fn decrease_balance(
    storage: &mut dyn Storage,
    address: &Addr,
    amount: Uint128,
) -> Result<Uint128, StdError> {
    let previous_balance = BALANCES.load(storage, address).unwrap_or_default();
    let new_balance = previous_balance.checked_sub(amount)?;
    BALANCES.save(storage, address, &new_balance)?;

    Ok(previous_balance)
}

/// Increase user balance and commit to store, returns previous balance
pub fn increase_balance(
    storage: &mut dyn Storage,
    address: &Addr,
    amount: Uint128,
) -> Result<Uint128, StdError> {
    let previous_balance = BALANCES.load(storage, address).unwrap_or_default();
    let new_balance = previous_balance + amount;
    BALANCES.save(storage, address, &new_balance)?;

    Ok(previous_balance)
}
