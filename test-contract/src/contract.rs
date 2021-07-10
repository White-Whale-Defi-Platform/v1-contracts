use cosmwasm_std::{
    to_binary, Api, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg
};
use terra_cosmwasm::{create_swap_msg, TerraRoute, TerraMsgWrapper};

use crate::pair::{HandleMsg as PairMsg};
use crate::asset::{Asset, AssetInfo};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, WhiteWhaleMsgWrapper, WhiteWhaleMsg};
use crate::state::{config, State, BURN_MINT_CONTRACT, LUNA_UST_PAIR};

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
// ) -> StdResult<HandleResponse<CosmosMsg>> {
    match msg {
        HandleMsg::Trade { amount } => try_trade(deps, env, amount)
    }
}
pub fn try_trade<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Coin,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
// ) -> StdResult<HandleResponse<CosmosMsg>> {

    // let offer = Asset{
    //     info: AssetInfo::NativeToken{ denom: String::from("uusd") },
    //     amount: amount.amount
    // };
    // let terraswap_msg = PairMsg::Swap{
    //     offer_asset: offer,
    //     belief_price: None,
    //     max_spread: None,
    //     to: None,
    // };
    // let csm_terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: HumanAddr::from(LUNA_UST_PAIR.clone()),
    //     send: vec![amount.clone()],
    //     msg: to_binary(&terraswap_msg)?,
    // });

    let ask_denom = "uluna".to_string();
    let swap_msg = create_swap_msg(
        env.contract.address,
        amount,
        ask_denom,
    );
    // let swap_msg = Swap {
    //     route: TerraRoute::Market,
    //     msg_data: WhiteWhaleMsg::Swap {
    //         trader,
    //         offer_coin,
    //         ask_denom,
    //     },
    // };
    // let csm_swap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: HumanAddr::from(BURN_MINT_CONTRACT.clone()),
    //     send: vec![],
    //     msg: to_binary(&swap_msg)?,
    // });
 
    Ok(HandleResponse {
        messages: vec![swap_msg],
        log: vec![],
        data: None,
    })
 
    // Ok(HandleResponse {
    //     messages: vec![csm_terraswap_msg],
    //     log: vec![],
    //     data: None,
    // })
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
