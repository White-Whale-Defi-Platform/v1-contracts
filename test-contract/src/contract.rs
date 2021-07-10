use cosmwasm_std::{
    to_binary, Api, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg
};

use crate::pair::{HandleMsg as PairMsg};
use crate::asset::{Asset, AssetInfo};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
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
) -> StdResult<HandleResponse<CosmosMsg>> {
    match msg {
        HandleMsg::Trade { amount, contract } => try_trade(deps, env, amount, contract)
    }
}
pub fn try_trade<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    amount: Coin,
    _contract: HumanAddr,
) -> StdResult<HandleResponse<CosmosMsg>> {

    let offer = Asset{
        info: AssetInfo::NativeToken{ denom: String::from("uusd") },
        amount: amount.amount
    };
    let msg = PairMsg::Swap{
        offer_asset: offer,
        belief_price: None,
        max_spread: None,
        to: None,
    };
    let exmes = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: HumanAddr::from(LUNA_UST_PAIR.clone()),
        send: vec![amount.clone()],
        msg: to_binary(&msg)?,
    });
 
     Ok(HandleResponse {
         messages: vec![exmes],
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
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
