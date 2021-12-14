use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use terraswap::asset::AssetInfo;
use white_whale::treasury::dapp_base::state::get_asset_info;

use crate::msg::AssetQueryResponse;

pub fn query_assets(deps: Deps, asset_names: Vec<String>) -> StdResult<Binary> {
    let mut assets: Vec<AssetInfo> = vec![];
    for asset in asset_names.into_iter() {
        assets.push(get_asset_info(deps, &asset)?);
    }

    to_binary(&AssetQueryResponse { assets })
}
