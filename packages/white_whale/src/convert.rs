use terraswap::asset::{Asset, AssetInfo};
use crate::denom::is_denom;
use cosmwasm_std::{Deps,StdResult};

pub fn convert_to_asset(deps: Deps, identifier: String) -> StdResult<AssetInfo> {
    return if is_denom(identifier.as_str()) {
        Ok(AssetInfo::NativeToken { denom: identifier })
    } else {
        deps.api.addr_validate(identifier.as_str())?;
        Ok(AssetInfo::Token { contract_addr: identifier})
    }
}