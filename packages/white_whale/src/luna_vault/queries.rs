use cosmwasm_std::{Addr, Decimal, Deps, QueryRequest, StdResult, to_binary, WasmQuery};

use crate::fee::{Fee, VaultFee};
use crate::luna_vault::msg::{FeeResponse, VaultQueryMsg};

/// Queries the luna vault fees
pub fn query_luna_vault_fees(
    deps: Deps,
    luna_vault_addr: &Addr,
) -> StdResult<VaultFee> {
    let response: FeeResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: luna_vault_addr.to_string(),
            msg: to_binary(&VaultQueryMsg::Fees {})?,
        }))?;

    Ok(response.fees)
}