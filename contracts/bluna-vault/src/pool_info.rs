use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    Api, CanonicalAddr, Decimal, Extern, HumanAddr,
    Querier, StdResult, Storage,
};
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw};

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfo {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: HumanAddr,
    pub liquidity_token: HumanAddr,
    pub slippage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfoRaw {
    pub asset_infos: [AssetInfoRaw; 2],
    pub contract_addr: CanonicalAddr,
    pub liquidity_token: CanonicalAddr,
    pub slippage: Decimal,
}

impl PoolInfoRaw {
    pub fn to_normal<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
    ) -> StdResult<PoolInfo> {
        Ok(PoolInfo {
            liquidity_token: deps.api.human_address(&self.liquidity_token)?,
            contract_addr: deps.api.human_address(&self.contract_addr)?,
            slippage: self.slippage,
            asset_infos: [
                self.asset_infos[0].to_normal(deps)?,
                self.asset_infos[1].to_normal(deps)?,
            ],
        })
    }

    pub fn query_pools<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
        contract_addr: &HumanAddr,
    ) -> StdResult<[Asset; 2]> {
        let info_0: AssetInfo = self.asset_infos[0].to_normal(deps)?;
        let info_1: AssetInfo = self.asset_infos[1].to_normal(deps)?;
        Ok([
            Asset {
                amount: info_0.query_pool(deps, contract_addr)?,
                info: info_0,
            },
            Asset {
                amount: info_1.query_pool(deps, contract_addr)?,
                info: info_1,
            },
        ])
    }
}
