use cosmwasm_std::{
    to_binary,Uint128,StdError, Binary, CosmosMsg, Env, DepsMut, Deps
    ,Response, StdResult,MessageInfo, WasmMsg,
};
use crate::state::{State, STATE, UST_DENOM};
use terraswap::querier::{query_token_balance};
use terraswap::asset::{Asset,AssetInfo};
use terraswap::pair::ExecuteMsg as HandleMsg;
use cw20::Cw20ExecuteMsg;
//use white_whale::msg::{create_terraswap_msg};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse};

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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Burn {} => buy_and_burn(deps.as_ref(), &env, info),
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

pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    to_binary(&ConfigResponse{
        owner: deps.api.addr_humanize(&state.owner_addr)?,
        token_addr: deps.api.addr_humanize(&state.whale_token_addr)?,
        pool_addr: deps.api.addr_humanize(&state.whale_pool_addr)?,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies,mock_env};
    use cosmwasm_std::{Api, from_binary};
    use terraswap::asset::AssetInfo;

    fn get_test_init_msg() -> InstantiateMsg {
        InstantiateMsg {
            whale_token_addr: "whale token".to_string(),
            whale_pair_addr: "terraswap pair".to_string(), 
            asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() },
        }
    }

    #[test]
    fn proper_initialization() {
        // Set dependencies, make the message, make the message info.
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        //Ssimulate transaction.
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        // TODO: implement query 
    }

    #[test]
    fn test_unauthorized_buy_and_burn() {
        // Set dependencies, make the message, make the message info.
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let creator_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        //Ssimulate transaction.
        let res = instantiate(deps.as_mut(), env.clone(), creator_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let attacker_info = MessageInfo{sender: deps.api.addr_validate("attacker").unwrap(), funds: vec![]};
        let ex_res = execute(deps.as_mut(), env, attacker_info, ExecuteMsg::Burn{});
        assert_eq!(ex_res.err(), Some(StdError::generic_err("Unauthorized.")))
    }

    #[test]
    fn test_no_ust_available(){
        // Set dependencies, make the message, make the message info.
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let creator_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        //Ssimulate transaction.
        let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let ex_res = execute(deps.as_mut(), env, creator_info, ExecuteMsg::Burn{});
        assert_eq!(ex_res.err(), Some(StdError::generic_err("No funds to buy token with.")))
    }

    #[test]
    fn test_config_query(){
        // Set dependencies, make the message, make the message info.
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let creator_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        //Ssimulate transaction.
        let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let q_res: ConfigResponse = from_binary(&query(deps.as_ref(), env, QueryMsg{}).unwrap()).unwrap();
        assert_eq!(q_res.owner,deps.api.addr_validate("creator").unwrap())
    }
}
