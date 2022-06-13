use cosmwasm_std::{to_binary, Addr, Deps, QueryRequest, StdResult, WasmQuery};

use crate::fee::VaultFee;
use crate::luna_vault::msg::{FeeResponse, VaultQueryMsg};

/// Queries the luna vault fees
pub fn query_luna_vault_fees(deps: Deps, luna_vault_addr: &Addr) -> StdResult<VaultFee> {
    let response: FeeResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: luna_vault_addr.to_string(),
        msg: to_binary(&VaultQueryMsg::Fees {})?,
    }))?;

    Ok(response.fees)
}
