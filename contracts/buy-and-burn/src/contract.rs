use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg,
    Uint128, WasmMsg, Reply, ReplyOn, entry_point
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use terraswap::querier::{query_balance, query_token_balance};
use white_whale::anchor::try_deposit_to_anchor_as_submsg;
use white_whale::msg::AnchorMsg;
use white_whale::query::anchor::query_aust_exchange_rate;
use std::str::FromStr;
use crate::error::BurnError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, ADMIN, STATE, UST_DENOM};


const BUY_WHALE_REPLY_ID: u64 = 1;
const ANCHOR_DEPOSIT_REPLY_ID: u64 = 2;
const ANCHOR_WITHDRAW_REPLY_ID: u64 = 3;


type BurnResult = Result<Response, BurnError>;


pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        whale_token_addr: deps.api.addr_canonicalize(&msg.whale_token_addr)?,
        whale_pool_addr: deps.api.addr_canonicalize(&msg.whale_pair_addr)?,
        anchor_money_market_addr: deps.api.addr_canonicalize(&msg.anchor_money_market_addr)?,
        aust_addr: deps.api.addr_canonicalize(&msg.aust_addr)?,
        deposits_in_uusd: Uint128::zero(),
        last_deposit_in_uusd: Uint128::zero(),
        anchor_deposit_threshold: msg.anchor_deposit_threshold,
        anchor_withdraw_threshold: msg.anchor_withdraw_threshold,
        anchor_deposit_ratio: msg.anchor_deposit_ratio
    };

    STATE.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> BurnResult {
    match msg {
        ExecuteMsg::Deposit {} => deposit_or_burn(deps, &env, info),
        ExecuteMsg::BurnProfits {} => try_burn_profits(deps, &env),
        ExecuteMsg::UpdateAdmin{ admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            Ok(ADMIN.execute_update_admin(deps, info, Some(admin_addr))?)
        },
        ExecuteMsg::UpdateAnchorDepositRatio { ratio } => set_anchor_deposit_ratio(deps, info, ratio),
        ExecuteMsg::UpdateAnchorDepositThreshold{ threshold } => set_anchor_deposit_threshold(deps, info, threshold),
        ExecuteMsg::UpdateAnchorWithdrawThreshold{ threshold } => set_anchor_withdraw_threshold(deps, info, threshold)
    }
}

pub fn set_anchor_deposit_ratio(deps: DepsMut, info: MessageInfo, ratio: Decimal) -> BurnResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    if ratio > Decimal::one() {
        return Err(BurnError::Std(StdError::generic_err("Ratio must be in [0, 1].")));
    }

    let mut state = STATE.load(deps.storage)?;
    state.anchor_deposit_ratio = ratio;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

pub fn set_anchor_deposit_threshold(deps: DepsMut, info: MessageInfo, threshold: Uint128) -> BurnResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut state = STATE.load(deps.storage)?;
    state.anchor_deposit_threshold = threshold;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

pub fn set_anchor_withdraw_threshold(deps: DepsMut, info: MessageInfo, threshold: Uint128) -> BurnResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut state = STATE.load(deps.storage)?;
    state.anchor_withdraw_threshold = threshold;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

pub fn try_buy_and_burn(deps: Deps, env: &Env) -> BurnResult {
    let state = STATE.load(deps.storage)?;
    let ust_amount = query_balance(&deps.querier, env.contract.address.clone(), UST_DENOM.to_string())?;
    if ust_amount == Uint128::zero() {
        return Err(BurnError::NotEnoughFunds{});
    }
    let mut offer = Asset {
        info: AssetInfo::NativeToken {
            denom: UST_DENOM.to_string(),
        },
        amount: ust_amount
    };
    let ust = offer.deduct_tax(&deps.querier)?;
    offer.amount = ust_amount;

    let buy_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.whale_pool_addr)?.to_string(),
        funds: vec![ust],
        msg: to_binary(&PairExecuteMsg::Swap {
            offer_asset: offer,
            belief_price: None,
            max_spread: None,
            to: None,
        })?,
    });

    Ok(Response::new().add_submessage(SubMsg {
        msg: buy_msg,
        gas_limit: None,
        id: BUY_WHALE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))

}

pub fn deposit(deps: DepsMut, env: &Env) -> BurnResult {
    let mut state = STATE.load(deps.storage)?;
    let mut deposit = deps.querier.query_balance(&env.contract.address, UST_DENOM)?;
    if deposit.amount < state.anchor_deposit_threshold {
        return Ok(Response::default());
    }

    deposit.amount = deposit.amount * state.anchor_deposit_ratio;
    state.last_deposit_in_uusd = deposit.amount;
    STATE.save(deps.storage, &state)?;
    let response = try_deposit_to_anchor_as_submsg(deps.api.addr_humanize(&state.anchor_money_market_addr)?.to_string(), deposit, ANCHOR_DEPOSIT_REPLY_ID)?;
    Ok(response)
}

pub fn deposit_or_burn(deps: DepsMut, env: &Env, msg_info: MessageInfo) -> BurnResult {
    if msg_info.funds.len() > 1 {
        return Err(BurnError::DepositTooManyTokens{});
    }
    if msg_info.funds[0].denom != UST_DENOM {
        return Err(BurnError::DepositOnlyUST{});
    }

    let state = STATE.load(deps.storage)?;
    let aust_value_in_ust = get_aust_value_in_ust(deps.as_ref(), env)?;
    if aust_value_in_ust < state.deposits_in_uusd + state.anchor_withdraw_threshold {
        return deposit(deps, env);
    }

    burn_profits(deps, aust_value_in_ust, msg_info.funds[0].amount)
}

pub fn get_aust_value_in_ust(deps: Deps, env: &Env) -> StdResult<Uint128> {
    let state = STATE.load(deps.storage)?;
    let aust_amount = query_token_balance(&deps.querier, deps.api.addr_humanize(&state.aust_addr)?, env.contract.address.clone())?;

    let epoch_state_response = query_aust_exchange_rate(deps, deps.api.addr_humanize(&state.anchor_money_market_addr)?.to_string())?;
    let aust_exchange_rate = Decimal::from_str(&epoch_state_response.exchange_rate.to_string())?;
    Ok(aust_exchange_rate*aust_amount)
}

pub fn burn_profits(deps: DepsMut, aust_value_in_ust: Uint128, deposit_amount: Uint128) -> BurnResult {
    let state = STATE.load(deps.storage)?;
    let withdraw_amount = aust_value_in_ust - state.deposits_in_uusd - deposit_amount;
    let withdraw_msg = CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: deps.api.addr_humanize(&state.aust_addr)?.to_string(),
        msg: to_binary(
            &Cw20ExecuteMsg::Send{
                contract: deps.api.addr_humanize(&state.anchor_money_market_addr)?.to_string(),
                amount: withdraw_amount,
                msg: to_binary(&AnchorMsg::RedeemStable{})?
            }
        )?,
        funds: vec![]
    });
    Ok(Response::new().add_submessage(SubMsg{
        msg: withdraw_msg,
        gas_limit: None,
        id: ANCHOR_WITHDRAW_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))

}

pub fn try_burn_profits(deps: DepsMut, env: &Env) -> BurnResult {
    let state = STATE.load(deps.storage)?;

    let aust_value_in_ust = get_aust_value_in_ust(deps.as_ref(), env)?;
    if aust_value_in_ust < state.deposits_in_uusd + state.anchor_withdraw_threshold {
        return Err(BurnError::Std(StdError::generic_err(format!("Not enough profits: {} - {} < {}", aust_value_in_ust, state.deposits_in_uusd, state.anchor_withdraw_threshold))));
    }
    burn_profits(deps, aust_value_in_ust, Uint128::zero())
}

pub fn burn_whale(deps: Deps, env: &Env) -> StdResult<CosmosMsg> {
    let state = STATE.load(deps.storage)?;
    let balance: Uint128 = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&state.whale_token_addr)?,
        env.contract.address.clone(),
    )?;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.whale_token_addr)?.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Burn { amount: balance })?,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> BurnResult {
    if msg.id == BUY_WHALE_REPLY_ID {
        return Ok(Response::default().add_message(burn_whale(deps.as_ref(), &env)?))
    }
    if msg.id == ANCHOR_DEPOSIT_REPLY_ID {
        let mut state = STATE.load(deps.storage)?;
        state.deposits_in_uusd += state.last_deposit_in_uusd;
        state.last_deposit_in_uusd = Uint128::zero();
        STATE.save(deps.storage, &state)?;
        return try_buy_and_burn(deps.as_ref(), &env);
    }
    if msg.id == ANCHOR_WITHDRAW_REPLY_ID {
        return try_buy_and_burn(deps.as_ref(), &env);
    }
    Ok(Response::default())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin{} => Ok(to_binary(&ADMIN.query_admin(deps)?)?),
        QueryMsg::Config{} => query_config(deps)
    }
}

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    to_binary(&ConfigResponse {
        token_addr: deps.api.addr_humanize(&state.whale_token_addr)?,
        pool_addr: deps.api.addr_humanize(&state.whale_pool_addr)?,
        anchor_money_market_addr: deps.api.addr_humanize(&state.anchor_money_market_addr)?,
        aust_addr: deps.api.addr_humanize(&state.aust_addr)?,
        anchor_deposit_threshold: state.anchor_deposit_threshold,
        anchor_withdraw_threshold: state.anchor_withdraw_threshold,
        anchor_deposit_ratio: state.anchor_deposit_ratio
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{from_binary, Api};
    use cw_controllers::AdminResponse;

    fn get_test_init_msg() -> InstantiateMsg {
        InstantiateMsg {
            whale_token_addr: "whale token".to_string(),
            whale_pair_addr: "terraswap pair".to_string(),
            anchor_money_market_addr: "anchor money market".to_string(),
            aust_addr: "aust".to_string(),
            anchor_deposit_threshold: Uint128::from(1000000000u64),
            anchor_withdraw_threshold: Uint128::from(1000000000u64),
            anchor_deposit_ratio: Decimal::from_ratio(1u64, 2u64)
        }
    }

    #[test]
    fn proper_initialization() {
        // Set dependencies, make the message, make the message info.
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        // Simulate transaction.
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        // TODO: implement query
    }

    #[test]
    fn test_set_anchor_deposit_ratio() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config{}).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_ne!(state.anchor_deposit_ratio, Decimal::percent(3u64));
        let _res = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateAnchorDepositRatio{ ratio: Decimal::percent(3u64) }).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.anchor_deposit_ratio, Decimal::percent(3u64));
    }

    #[test]
    fn test_set_anchor_deposit_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config{}).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_ne!(state.anchor_deposit_threshold, Uint128::from(3u64));
        let _res = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateAnchorDepositThreshold{ threshold: Uint128::from(3u64) }).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.anchor_deposit_threshold, Uint128::from(3u64));
    }

    #[test]
    fn test_set_anchor_withdraw_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config{}).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_ne!(state.anchor_withdraw_threshold, Uint128::from(3u64));
        let _res = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateAnchorWithdrawThreshold{ threshold: Uint128::from(3u64) }).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.anchor_withdraw_threshold, Uint128::from(3u64));
    }

    #[test]
    fn test_anchor_deposit_ratio_can_not_be_greater_than_one() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config{}).unwrap();
        let res = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateAnchorDepositRatio{ ratio: Decimal::percent(101u64) });
        match res {
            Err(_) => {},
            Ok(_) => panic!("unexpected")
        }
    }

    #[test]
    fn test_only_owner_can_change_anchor_deposit_ratio() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };
        let other_info = MessageInfo {
            sender: deps.api.addr_validate("other").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config{}).unwrap();
        let res = execute(deps.as_mut(), env, other_info, ExecuteMsg::UpdateAnchorDepositRatio{ ratio: Decimal::percent(3u64) });
        match res {
            Err(_) => {},
            Ok(_) => panic!("unexpected")
        }
    }

    #[test]
    fn test_only_owner_can_change_anchor_deposit_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };
        let other_info = MessageInfo {
            sender: deps.api.addr_validate("other").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config{}).unwrap();
        let res = execute(deps.as_mut(), env, other_info, ExecuteMsg::UpdateAnchorDepositThreshold{ threshold: Uint128::from(3u64) });
        match res {
            Err(_) => {},
            Ok(_) => panic!("unexpected")
        }
    }

    #[test]
    fn test_only_owner_can_change_anchor_withdraw_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };
        let other_info = MessageInfo {
            sender: deps.api.addr_validate("other").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config{}).unwrap();
        let res = execute(deps.as_mut(), env, other_info, ExecuteMsg::UpdateAnchorWithdrawThreshold{ threshold: Uint128::from(3u64) });
        match res {
            Err(_) => {},
            Ok(_) => panic!("unexpected")
        }
    }

    #[test]
    fn test_config_query() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let creator_info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let q_res: ConfigResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(q_res.token_addr, deps.api.addr_validate("whale token").unwrap())
    }

    #[test]
    fn test_admin_query() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let creator_info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let q_res: AdminResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::Admin {}).unwrap()).unwrap();
        assert_eq!(q_res.admin.unwrap(), deps.api.addr_validate("creator").unwrap())
    }
}
