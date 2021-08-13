use cosmwasm_std::{
    to_binary,Uint128,StdError, Binary, CosmosMsg, Env, DepsMut, Deps
    ,Response, StdResult,MessageInfo, WasmMsg
};
use crate::state::{State, STATE, UST_DENOM};
use terraswap::querier::{query_token_balance};
use terraswap::asset::{Asset,AssetInfo};
use terraswap::pair::ExecuteMsg as HandleMsg;
use cw20::Cw20ExecuteMsg;
//use white_whale::msg::{create_terraswap_msg};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    let state = State {
        owner_addr: deps.api.addr_canonicalize(info.sender.as_str())?,
        whale_token_addr: deps.api.addr_canonicalize(&msg.whale_token_addr)?,
        whale_pool_addr: deps.api.addr_canonicalize(&msg.whale_pair_addr)?,
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

pub fn execute(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Burn {} => buy_and_burn(deps, &env, info),
    }
}

pub fn buy_and_burn(deps: Deps, env: &Env, msg_info: MessageInfo,) -> StdResult<Response>{
    let buy_msg = buy_whale(deps, &env, msg_info)?;
    let burn_msg = burn_whale(deps, &env)?;
    Ok(Response::new().add_message(buy_msg).add_message(burn_msg)
    )
}

pub fn burn_whale(deps: Deps, env: &Env) -> StdResult<CosmosMsg> {
    let state = STATE.load(deps.storage)?;
    let balance: Uint128 = query_token_balance(&deps.querier, deps.api.addr_humanize(&state.whale_token_addr)?, env.contract.address.clone())?;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: state.whale_token_addr.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: balance 
        })?
    }))
}

pub fn buy_whale(deps: Deps, env: &Env, msg_info: MessageInfo,) -> StdResult<CosmosMsg>{
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.owner_addr {
        return Err(StdError::generic_err("Unauthorized."));
    }
    let ust = deps.querier.query_balance(&env.contract.address,UST_DENOM)?;
    if ust.amount == Uint128::zero() {
        return Err(StdError::generic_err("No funds to buy token with."));
    }
    let offer = Asset{
        info: AssetInfo::NativeToken{ denom: ust.denom.clone() },
        amount: ust.amount
    };

    //We don't care about slippage
    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.whale_pool_addr.to_string(),
        funds: vec![ust.clone()],
        msg: to_binary(&HandleMsg::Swap{
            offer_asset: offer,
            belief_price: None,
            max_spread: None,
            to: None,
        })?,
    });
    return Ok(terraswap_msg)
}

pub fn query(_deps: DepsMut, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::{from_binary, Coin, coins, HumanAddr};
//     use cosmwasm_std::testing::{mock_dependencies, mock_env};


//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(20, &[]);
//         let whale_token_addr = HumanAddr::from("test_vault");
//         let owner_addr = HumanAddr::from("owner");
//         println!("Whale token addr is {:?}.", whale_token_addr);
//         println!("Owner addr is {:?}.", owner_addr);
//         let msg = InitMsg {
//             owner_addr : owner_addr.clone(),
//             whale_token_addr : whale_token_addr.clone(),
//         };
//         let env = mock_env("creator", &coins(1000, "earth"));

//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());
//     }
// }
