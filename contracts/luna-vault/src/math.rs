use cosmwasm_std::{Decimal, Uint128};

const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000u128);

/// decimal_division returns a / b. Panics if b is zero.
pub fn decimal_division(a: Uint128, b: Decimal) -> Uint128 {
    let decimal = Decimal::from_ratio(a, b * DECIMAL_FRACTIONAL);
    decimal * DECIMAL_FRACTIONAL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_division_by_non_zero() {
        let a = Uint128::new(100);
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = decimal_division(a, b);
        assert_eq!(res, Uint128::new(500));
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn test_decimal_division_by_zero() {
        let a = Uint128::new(100);
        let b = Decimal::from_ratio(Uint128::new(0), Uint128::new(50));
        decimal_division(a, b);
    }
}
