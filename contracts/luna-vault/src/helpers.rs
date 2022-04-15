use cosmwasm_std::{Decimal, StdError, StdResult};

pub fn validate_rate(rate: Decimal) -> StdResult<Decimal> {
    if rate > Decimal::one() {
        return Err(StdError::generic_err(format!(
            "Rate can not be bigger than one (given value: {})",
            rate
        )));
    }

    Ok(rate)
}

