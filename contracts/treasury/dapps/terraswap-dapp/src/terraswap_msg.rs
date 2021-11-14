use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Deps, Empty, StdResult, WasmMsg,
};

use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairMsg;

pub fn deposit_lp_msg(
    deps: Deps,
    assets: [Asset; 2],
    pair_addr: Addr,
) -> StdResult<Vec<CosmosMsg<Empty>>> {
    let mut msgs: Vec<CosmosMsg<Empty>> = vec![];
    let mut coins: Vec<Coin> = vec![];
    for asset in assets.iter() {
        match &asset.info {
            AssetInfo::Token { contract_addr } => {
                msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                        spender: pair_addr.to_string(),
                        amount: asset.amount,
                        expires: None,
                    })?,
                    funds: vec![],
                }));
            }
            AssetInfo::NativeToken { .. } => coins.push(asset.deduct_tax(&deps.querier)?),
        }
    }

    let lp_msg = PairMsg::ProvideLiquidity {
        assets,
        slippage_tolerance: None,
        receiver: None,
    };

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pair_addr.to_string(),
        msg: to_binary(&lp_msg)?,
        funds: coins,
    }));

    Ok(msgs)
}
