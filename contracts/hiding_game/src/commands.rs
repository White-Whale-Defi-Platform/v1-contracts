use std::cmp::max;
use std::collections::HashMap;
use std::ops::Index;

use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal, Deps, Env, Fraction, MessageInfo, Response, Uint128,
    WasmMsg, QuerierWrapper, Addr, QueryRequest, WasmQuery, StdResult,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::Asset;
use terraswap::pair::ExecuteMsg as PairExecuteMsg;

use terraswap::pair::{Cw20HookMsg, PoolResponse};

use white_whale::query::terraswap::{query_asset_balance, query_pool};
use white_whale::treasury::dapp_base::common::PAIR_POSTFIX;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::state::BASESTATE;
use white_whale::treasury::msg::send_to_treasury;
use white_whale::treasury::vault_assets::get_identifier;

use crate::contract::HidingGameResult;
use crate::error::HidingGameError;
use crate::state::CONFIG;
use crate::terraswap_msg::asset_into_swap_msg;

const VARIANTS: [&str; 3] = ["astro", "tswap", "loop"];


/// Function constructs terraswap swap messages and forwards them to the treasury
#[allow(clippy::too_many_arguments)]
pub fn whale_trade(
    deps: Deps,
    _env: Env,
    msg_info: MessageInfo,
    pair: (String, String),
    offer_id: String,
    amount: Uint128,
    max_spread: Option<Decimal>,
    belief_price: Option<Decimal>,
) -> HidingGameResult {
    let config = CONFIG.load(deps.storage)?;

    let pool_id = construct_pool_id(&pair);

    let pool_variants = construct_pool_variants(&pool_id);
    let pair_addresses = config.memory.try_query_contracts(deps, &pool_variants);

    if pair_addresses.len() == 0 {
        return Err(HidingGameError::NoRegisteredPair(pool_id));
    }

    let offer_asset_info = config.memory.query_asset(deps, &offer_id)?;

    let asset = Asset{
        info: offer_asset_info,
        amount
    };

    let mut sim_results = vec![];
    // (pool_id, 
    let mut best_pool: (&str, u128) = ("", 0);
    // search best pool to trade
    for (id, pair) in pair_addresses.iter() {
        let res = swap_simulation(&deps.querier, pair,asset.clone())?;
        if res > best_pool.1 {
            best_pool = (id, res);
        }
    }

    let msg = match asset.info {
        terraswap::asset::AssetInfo::Token { contract_addr: token_addr } => {
            let cw_msg = 
            Cw20ExecuteMsg::SendFrom{
                owner: msg_info.sender.into_string(),
                amount: asset.amount,
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset: asset,
                    belief_price,
                    max_spread,
                    to: None,
                })?,
                contract: pair_addresses.get(best_pool.0).unwrap().to_string(),
            };
            
            // call on cw20
            CosmosMsg::Wasm(WasmMsg::Execute{
                contract_addr: token_addr,
                msg: to_binary(&cw_msg)?,
                funds: vec![]
            })
        },
        terraswap::asset::AssetInfo::NativeToken { denom } => {
        // we received the native tokens so we need to swap
        asset_into_swap_msg(
            deps,
            pair_addresses.get(best_pool.0).unwrap().to_owned(),
            asset,
            max_spread,
            belief_price,
            // Msg is executed by us but caller should get return
            Some(msg_info.sender.to_string()),
        )?
    }
};    

Ok(Response::new().add_message(msg).add_message(msg))
}


// see if arb opportunity is created
pub fn after_trade(deps: cosmwasm_std::DepsMut, env: Env, info: MessageInfo, ) -> HidingGameResult {

}


fn construct_pool_variants(pool_id: &str) -> Vec<String> {
    VARIANTS.iter().map(|dex| format!("{}_{}", *dex, pool_id)).collect()
}

fn construct_pool_id(pair: &(String, String)) -> String {
    if pair.0.gt(&pair.1) {
        format!("{}_{}", pair.1, pair.0)
    } else {
        format!("{}_{}", pair.0, pair.1)
    }
}

fn swap_simulation(querier: &QuerierWrapper , pair_addr: &Addr, offer_asset: Asset ) -> StdResult<u128> {
    let response: terraswap::pair::SimulationResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pair_addr.to_string(),
            msg: to_binary(&terraswap::pair::QueryMsg::Simulation{
                offer_asset
            })?,
        }))?;
    Ok(response.return_amount.u128())
}
