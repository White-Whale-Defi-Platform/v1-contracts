
use cosmwasm_std::{CosmosMsg, Deps, StdResult};
use cosmwasm_std::{Addr, CanonicalAddr, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


use terraswap::asset::{Asset};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
    pub share: Decimal,
}

impl Fee {
    pub fn compute(&self, amount: Uint128) -> Uint128 {
        amount * self.share
    }

    pub fn msg(&self, deps: Deps, asset: Asset, recipient: Addr) -> StdResult<CosmosMsg> {
        Ok(asset.into_msg(&deps.querier, recipient)?)
    }
}

// #[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CappedFee {
//     pub fee: Fee,
//     pub max_fee: Uint128,
// }

// impl CappedFee {
//     pub fn compute(&self, value: Uint128) -> Uint128 {
//         min(self.fee.compute(value), self.max_fee)
//     }

//     pub fn msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
//         &self,
//         deps: Deps,
//         value: Uint128,
//         denom: String,
//         address: String,
//     ) -> StdResult<CosmosMsg<T>> {
//         let fee = self.compute(value);
//         let community_fund_asset = Asset {
//             info: AssetInfo::NativeToken { denom },
//             amount: fee,
//         };

//         Ok(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: address,
//             funds: vec![community_fund_asset.deduct_tax(&deps.querier)?],
//             msg: to_binary(&CommunityFundMsg::Deposit {})?,
//         }))
//     }
// }

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultFee {
    pub flash_loan_fee: Fee,
    pub warchest_fee: Fee,
    pub warchest_addr: CanonicalAddr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee() {
        let fee = Fee {
            share: Decimal::percent(20u64),
        };
        let deposit = Uint128::from(1000000u64);
        let deposit_fee = fee.compute(deposit);
        assert_eq!(deposit_fee, Uint128::from(200000u64));
    }

    // #[test]
    // fn test_capped_fee() {
    //     let max_fee = Uint128::from(1000u64);
    //     let fee = CappedFee {
    //         fee: Fee {
    //             share: Decimal::percent(20u64),
    //         },
    //         max_fee,
    //     };
    //     let deposit = Uint128::from(1000000u64);
    //     let deposit_fee = fee.compute(deposit);
    //     assert_eq!(deposit_fee, max_fee);
    // }
}
