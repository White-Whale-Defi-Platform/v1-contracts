use cosmwasm_std::{  to_binary, Coin, CosmosMsg, StdResult, WasmMsg };
use cosmwasm_std::{Deps, Uint128};
use crate::tax::{deduct_tax, compute_tax };




/// @dev Returns a Cosmos Msg to sell an mAsset (CW20)  via terraswap
pub fn trade_cw20_on_terraswap(
    pair_address: String,
    masset_to_sell_addr: String,
    amount: Uint128
) -> StdResult<CosmosMsg> {

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:pair_address,
        funds: vec![],
        msg: to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: terraswap::asset::Asset {
                info: terraswap::asset::AssetInfo::Token { contract_addr: masset_to_sell_addr  },
                amount: amount
            } ,
            belief_price: None,
            max_spread: None,
            to: None
        })?,
    }))
}




/// @dev Returns a Cosmos Msg to sell a Native asset for UST  via terraswap
pub fn trade_native_on_terraswap(
    deps: Deps,    
    pair_address: String,
    native_denom: String,
    amount: Uint128
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:pair_address,
        funds: vec![deduct_tax(  deps,  Coin {   denom: native_denom.to_string(),  amount: amount.into(),}, )? ],
        msg: to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: terraswap::asset::Asset {
                info: terraswap::asset::AssetInfo::NativeToken { denom: native_denom  },
                amount: amount
            } ,
            belief_price: None,
            max_spread: None,
            to: None
        })?,
    }))
}