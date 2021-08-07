use cosmwasm_std::{to_binary, Api, Coin, Extern, HumanAddr, StdResult, Storage, Querier, QueryRequest, Uint128, WasmQuery};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{QueryMsg, SimulationResponse};


pub fn simulate_swap<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pool_address: HumanAddr,
    offer_coin: Coin
) -> StdResult<Uint128> {
    let response: SimulationResponse = deps.querier.query(
        &QueryRequest::Wasm(WasmQuery::Smart{
            contract_addr: pool_address,
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
