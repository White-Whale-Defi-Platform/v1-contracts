use cosmwasm_std::{
    to_binary, Api, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg, Uint128
};
use terra_cosmwasm::{create_swap_msg, TerraMsgWrapper};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, create_terraswap_msg};
use crate::state::{config, State, LUNA_UST_PAIR};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    match msg {
        HandleMsg::AbovePeg { amount, luna_price } => try_arb_above_peg(deps, env, amount, luna_price),
        HandleMsg::BelowPeg { amount, luna_price } => try_arb_below_peg(deps, env, amount, luna_price)
    }
}

pub fn try_arb_below_peg<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Coin,
    _luna_price: Coin,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {

    let ask_denom = "uluna".to_string();

    let swap_msg = create_swap_msg(
        env.contract.address,
        amount.clone(),
        ask_denom.clone(),
    );
    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: HumanAddr::from(LUNA_UST_PAIR.clone()),
        send: vec![Coin{ denom: ask_denom.clone(), amount: Uint128(5000)}],
        msg: to_binary(&create_terraswap_msg(ask_denom.clone()))?,
    });

    Ok(HandleResponse {
        messages: vec![swap_msg, terraswap_msg],
        log: vec![],
        data: None,
    })
}

pub fn try_arb_above_peg<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Coin,
    luna_price: Coin,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {

    let ask_denom = "uusd".to_string();
    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: HumanAddr::from(LUNA_UST_PAIR.clone()),
        send: vec![amount.clone()],
        msg: to_binary(&create_terraswap_msg(ask_denom.clone()))?,
    });

    let swap_msg = create_swap_msg(
        env.contract.address,
        Coin{denom: "uluna".to_string(), amount: amount.amount.clone().multiply_ratio(1u128, luna_price.amount.0)},
        ask_denom.clone(),
    );

    Ok(HandleResponse {
        messages: vec![terraswap_msg, swap_msg],
        log: vec![],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _msg: QueryMsg,
) -> StdResult<Binary> {
    Err(StdError::generic_err("not implemented"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins};
    use terra_cosmwasm::TerraRoute;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn when_given_a_below_peg_msg_then_handle_returns_first_a_mint_then_a_terraswap_msg() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = HandleMsg::BelowPeg {
            amount: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
            luna_price: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)}
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(2, res.messages.len());
        let first_msg = res.messages[0].clone();
        match first_msg {
            CosmosMsg::Bank(_bank_msg) => assert_eq!(true, false),
            CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
            CosmosMsg::Staking(_staking_msg) => assert_eq!(true, false),
            CosmosMsg::Wasm(_wasm_msg) => assert_eq!(true, false)
        }
        let second_msg = res.messages[1].clone();
        match second_msg {
            CosmosMsg::Bank(_bank_msg) => assert_eq!(true, false),
            CosmosMsg::Custom(_t) => assert_eq!(true, false),
            CosmosMsg::Staking(_staking_msg) => assert_eq!(true, false),
            CosmosMsg::Wasm(_wasm_msg) => {}
        }
    }

    #[test]
    fn when_given_an_above_peg_msg_then_handle_returns_first_a_terraswap_then_a_mint_msg() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = HandleMsg::AbovePeg {
            amount: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
            luna_price: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)}
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(2, res.messages.len());
        let first_msg = res.messages[0].clone();
        match first_msg {
            CosmosMsg::Bank(_bank_msg) => assert_eq!(true, false),
            CosmosMsg::Custom(_t) => assert_eq!(true, false),
            CosmosMsg::Staking(_staking_msg) => assert_eq!(true, false),
            CosmosMsg::Wasm(_wasm_msg) => {}
        }
        let second_msg = res.messages[1].clone();
        match second_msg {
            CosmosMsg::Bank(_bank_msg) => assert_eq!(true, false),
            CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
            CosmosMsg::Staking(_staking_msg) => assert_eq!(true, false),
            CosmosMsg::Wasm(_wasm_msg) => assert_eq!(true, false)
        }
    }
}
