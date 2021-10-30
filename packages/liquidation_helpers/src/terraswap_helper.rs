use cosmwasm_std::{  to_binary, Coin, CosmosMsg, StdResult, WasmMsg };
use cosmwasm_std::{Deps, Uint128};
use crate::tax::{deduct_tax, compute_tax };
use crate::helper::build_send_cw20_token_msg;


/// @dev Returns a Cosmos Msg to sell an mAsset (CW20)  via terraswap
pub fn trade_cw20_on_terraswap(
    pair_address: String,
    asset_to_sell_addr: String,
    amount: Uint128
) -> StdResult<CosmosMsg> {
    let binary_msg = to_binary(&terraswap::pair::Cw20HookMsg::Swap {
        belief_price: None,
        max_spread: None,
        to: None,
        })?;
    Ok(build_send_cw20_token_msg(pair_address, asset_to_sell_addr, amount, binary_msg )?)
}




/// @dev Returns a Cosmos Msg to sell a Native asset for UST  via terraswap
pub fn trade_native_on_terraswap(
    deps: Deps,    
    pair_address: String,
    native_denom: String,
    amount: Uint128
) -> StdResult<CosmosMsg> {
    let tax = compute_tax(deps, &Coin { denom: native_denom.clone(), amount: amount, } )?;
    let amount_received_by_pair = amount - tax;
    // let amount_ = deduct_tax(  deps,  Coin {   denom: native_denom.to_string(),  amount: amount.into(),}, )? 
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:pair_address,
        funds: vec![Coin {
            denom: native_denom.clone(),
            amount: amount_received_by_pair,
        }],
        msg: to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: terraswap::asset::Asset {
                info: terraswap::asset::AssetInfo::NativeToken { denom: native_denom  },
                amount: amount_received_by_pair
            } ,
            belief_price: None,
            max_spread: None,
            to: None
        })?,
    }))
}