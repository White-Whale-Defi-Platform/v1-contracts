use cosmwasm_std::{to_binary, Addr, Coin, Deps, StdResult, QueryRequest, Uint128, WasmQuery};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{QueryMsg, SimulationResponse};


pub fn simulate_swap(
    deps: Deps,
    pool_address: Addr,
    offer_coin: Coin
) -> StdResult<Uint128> {
    let response: SimulationResponse = deps.querier.query(
        &QueryRequest::Wasm(WasmQuery::Smart{
            contract_addr: pool_address.to_string(),
            msg: to_binary(&QueryMsg::Simulation{
                offer_asset: Asset{
                    info: AssetInfo::NativeToken{ denom: offer_coin.denom },
                    amount: offer_coin.amount,
                }
            })?
        })
    )?;

    Ok(response.return_amount)
}
