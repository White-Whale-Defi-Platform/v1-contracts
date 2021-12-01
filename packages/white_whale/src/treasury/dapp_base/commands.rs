//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};
use crate::treasury::dapp_base::common::DAppResult;
use crate::treasury::dapp_base::state::{ADDRESS_BOOK, ADMIN, STATE};

pub fn update_address_book(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(String, String)>,
    to_remove: Vec<String>,
) -> DAppResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (name, new_address) in to_add.into_iter() {
        // update function for new or existing keys
        let insert = |vault_asset: Option<String>| -> StdResult<String> {
            match vault_asset {
                // Todo: is there a better way to just leave the data untouched?
                Some(present) => Ok(present),
                None => Ok(new_address),
            }
        };
        ADDRESS_BOOK.update(deps.storage, name.as_str(), insert)?;
    }

    for name in to_remove {
        ADDRESS_BOOK.remove(deps.storage, name.as_str());
    }

    Ok(Response::new().add_attribute("action", "updated address book"))
}

/// Updates trader or treasury address
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    treasury_address: Option<String>,
    trader: Option<String>,
) -> DAppResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = STATE.load(deps.storage)?;

    if let Some(treasury_address) = treasury_address {
        state.treasury_address = deps.api.addr_canonicalize(&treasury_address)?;
    }

    if let Some(trader) = trader {
        state.trader = deps.api.addr_canonicalize(&trader)?;
    }

    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("Update:", "Successfull"))
}
