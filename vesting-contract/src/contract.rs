use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg, Uint128
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, BalanceResponse, RecipientResponse, VestingScheduleResponse};
use crate::state::{config, config_read, State};
use cw20::{Cw20HandleMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        recipient: deps.api.canonical_address(&msg.recipient)?,
        token_contract: deps.api.canonical_address(&msg.token_contract)?,
        vesting_schedule: msg.vesting_schedule,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Claim { amount } => try_claim(deps, env, amount),
    }
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128
) -> StdResult<HandleResponse> {
    let state = config_read(&deps.storage).load()?;
    if env.message.sender != deps.api.human_address(&state.recipient)? {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let mut response = HandleResponse::default();
    if env.block.height > state.vesting_schedule {
        response.messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&state.token_contract)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: deps.api.human_address(&state.recipient)?,
                amount: amount,
            })?,
            send: vec![],
        }));
    } else {
        let remaining = state.vesting_schedule - env.block.height;
        return Err(StdError::generic_err("Funds not vested yet: ".to_string() + &remaining.to_string() + " blocks remaining"));
    }

    Ok(response)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        // QueryMsg::Balance{} => to_binary(&try_query_balance(deps)?),
        QueryMsg::Recipient{} => to_binary(&try_query_recipient(deps)?),
        QueryMsg::VestingSchedule{} => to_binary(&try_query_vesting_schedule(deps)?),
    }
}

// fn try_query_balance<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<BalanceResponse> {
//     let state = config_read(&deps.storage).load()?;
//     Ok(BalanceResponse { amount: "123uusd" })
// }

fn try_query_recipient<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<RecipientResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(RecipientResponse { recipient: deps.api.human_address(&state.recipient)? })
}

fn try_query_vesting_schedule<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<VestingScheduleResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(VestingScheduleResponse { vesting_schedule: state.vesting_schedule })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, HumanAddr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let recipient = HumanAddr::from("test1");
        let msg = InitMsg { recipient: recipient.clone(), token_contract: HumanAddr::from("test2"), vesting_schedule: 1u64 };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::Recipient {}).unwrap();
        let value: RecipientResponse = from_binary(&res).unwrap();
        assert_eq!(recipient, value.recipient);
    }

    #[test]
    fn claim_successfully() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let recipient = HumanAddr::from("test1");
        let msg = InitMsg { recipient: recipient.clone(), token_contract: HumanAddr::from("test2"), vesting_schedule: 1u64 };
        let env = mock_env("test1", &coins(2, "token"));
        let _res = init(&mut deps, env.clone(), msg).unwrap();

        // beneficiary can release it
        let msg = HandleMsg::Claim { amount: Uint128(100)};
        let res = handle(&mut deps, env, msg);
        match res {
            Ok(_) => {},
            Err(_) => panic!("expected successful claim")
        };
    }

    #[test]
    fn claim_fails_if_vesting_time_did_not_expire() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let recipient = HumanAddr::from("test1");
        let msg = InitMsg { recipient: recipient.clone(), token_contract: HumanAddr::from("test2"), vesting_schedule: 12346u64 };
        let env = mock_env("test1", &coins(2, "token"));
        let _res = init(&mut deps, env.clone(), msg).unwrap();

        let msg = HandleMsg::Claim { amount: Uint128(100)};
        let res = handle(&mut deps, env, msg);
        match res {
            Ok(_) => panic!("expected error"),
            Err(_) => {}
        };
    }

    #[test]
    fn claim_fails_for_unauthorized_accounts() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let recipient = HumanAddr::from("test1");
        let msg = InitMsg { recipient: recipient.clone(), token_contract: HumanAddr::from("test2"), vesting_schedule: 1u64 };
        let env = mock_env("test1", &coins(2, "token"));
        let _res = init(&mut deps, env.clone(), msg).unwrap();

        let env = mock_env("test3", &coins(2, "token"));
        let msg = HandleMsg::Claim { amount: Uint128(100)};
        let res = handle(&mut deps, env, msg);
        match res {
            Ok(_) => panic!("expected error"),
            Err(_) => {}
        };
    }
}
