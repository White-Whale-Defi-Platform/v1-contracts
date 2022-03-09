use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, Empty, StdResult, WasmMsg};

use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use white_whale::tax::compute_tax;

// Adapted from terraswap_router operations.rs
/// Constructs a swap msg
pub fn asset_into_swap_msg(
    deps: Deps,
    pair_contract: Addr,
    offer_asset: Asset,
    max_spread: Option<Decimal>,
    belief_price: Option<Decimal>,
    to: Option<String>,
) -> StdResult<CosmosMsg<Empty>> {
    match offer_asset.info.clone() {
        AssetInfo::NativeToken { denom } => {
            // deduct tax first
            let amount = offer_asset.amount.checked_sub(compute_tax(
                deps,
                &Coin::new(offer_asset.amount.u128(), denom.clone()),
            )?)?;

            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_contract.to_string(),
                funds: vec![Coin { denom, amount }],
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset: Asset {
                        amount,
                        ..offer_asset
                    },
                    belief_price,
                    max_spread,
                    to,
                })?,
            }))
        }
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair_contract.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset,
                    belief_price,
                    max_spread,
                    to,
                })?,
            })?,
        })),
    }
}
