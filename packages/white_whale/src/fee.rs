use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, Decimal, Uint128};
use std::cmp::min;


#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
    pub share: Decimal
}

impl Fee {
    pub fn compute(&self, value: Uint128) -> Uint128 {
        value*self.share
    }
}


#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CappedFee {
    pub fee: Fee,
    pub max_fee: Uint128
}

impl CappedFee {
    pub fn compute(&self, value: Uint128) -> Uint128 {
        min(self.fee.compute(value), self.max_fee)
    }
}


#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultFee {
    pub community_fund_fee: CappedFee,
    pub warchest_fee: Fee,
    pub community_fund_addr: CanonicalAddr,
    pub warchest_addr: CanonicalAddr
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee() {
        let fee = Fee{share: Decimal::percent(20u64)};
        let deposit = Uint128::from(1000000u64);
        let deposit_fee = fee.compute(deposit);
        assert_eq!(deposit_fee, Uint128::from(200000u64));
    }

    #[test]
    fn test_capped_fee() {
        let max_fee = Uint128::from(1000u64);
        let fee = CappedFee{fee: Fee{share: Decimal::percent(20u64)}, max_fee};
        let deposit = Uint128::from(1000000u64);
        let deposit_fee = fee.compute(deposit);
        assert_eq!(deposit_fee, max_fee);
    }
}