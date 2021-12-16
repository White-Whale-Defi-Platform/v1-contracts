use cosmwasm_std::{Deps, StdResult};
use terraswap::asset::AssetInfo;
use white_whale::denom::is_denom;
use white_whale::treasury::dapp_base::state::ADDRESS_BOOK;

// Returns the asset info for an address book entry.
pub fn get_asset_info(deps: Deps, id: &str) -> StdResult<AssetInfo> {
    let address_or_denom = ADDRESS_BOOK.load(deps.storage, id)?;
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
