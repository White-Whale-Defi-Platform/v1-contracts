use cosmwasm_std::{ Binary, Deps, StdResult, WasmQuery, QueryRequest, Addr };

use terraswap::asset::AssetInfo;
use crate::denom::is_denom;
use cosmwasm_storage::to_length_prefixed;


pub fn query_assets_from_mem(deps: Deps, memory_addr: Addr, asset_names: Vec<String>) -> StdResult<Vec<AssetInfo>> {
    let mut assets: Vec<String> = vec![];
        
    for asset in asset_names.into_iter() {
        assets.push(
            deps
            .querier
            .query(&QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: memory_addr.to_string(),
                key: Binary::from(concat(
                    &to_length_prefixed(b"address_book"),
                    asset.as_bytes(),
                )),
            }))?);
    }
    let assets_as_info: Vec<AssetInfo> = assets.iter().map(|a| to_asset_info(deps,a).unwrap()).collect();
    Ok(assets_as_info)
}


// Returns the asset info for an address book entry.
pub fn to_asset_info(deps: Deps, address_or_denom: &str) -> StdResult<AssetInfo> {
    return if is_denom(address_or_denom) {
        Ok(AssetInfo::NativeToken {
            denom: String::from(address_or_denom),
        })
    } else {
        deps.api.addr_validate(address_or_denom)?;
        Ok(AssetInfo::Token {
            contract_addr: String::from(address_or_denom),
        })
    };
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}