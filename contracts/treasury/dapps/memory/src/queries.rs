use cosmwasm_std::{to_binary, Binary, Deps, StdResult, WasmQuery, QueryRequest};

use cw_storage_plus::Map;
use terraswap::asset::AssetInfo;
use white_whale::treasury::dapp_base::state::{get_asset_info, ADDRESS_BOOK};
use cosmwasm_storage::to_length_prefixed;

use crate::msg::AssetQueryResponse;

pub fn query_assets(deps: Deps, asset_names: Vec<String>) -> StdResult<Binary> {
    let mut assets: Vec<AssetInfo> = vec![];
    for asset in asset_names.into_iter() {
        assets.push(get_asset_info(deps, &asset)?);
    }

    to_binary(&AssetQueryResponse { assets })
}

pub fn query_assets_from_mem(deps: Deps, memory_addr: Addr, asset_names: Vec<String>) -> StdResult<Vec<AssetInfo>> {
    let mut assets: Vec<AssetInfo> = vec![];
        
    for asset in asset_names.into_iter() {
        assets.push(deps
            .querier
            .query(&QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: memory_addr.to_string(),
                key: Binary::from(concat(
                    &to_length_prefixed(b"address_book"),
                    asset.as_bytes(),
                )),
            }))?);
    }
    Ok(assets)
}

// Returns the asset info for an address book entry.
pub fn to_asset_info(deps: Deps, address_or_denom: &str) -> StdResult<AssetInfo> {
    return if is_denom(address_or_denom.as_str()) {
        Ok(AssetInfo::NativeToken {
            denom: address_or_denom,
        })
    } else {
        deps.api.addr_validate(address_or_denom.as_str())?;
        Ok(AssetInfo::Token {
            contract_addr: address_or_denom,
        })
    };
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}