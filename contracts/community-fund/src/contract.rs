use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use white_whale::community_fund::msg::{ConfigResponse, ExecuteMsg, QueryMsg};

use crate::commands;
use crate::error::CommunityFundError;
use crate::msg::InstantiateMsg;
use crate::state::{State, ADMIN, STATE};

/*
    The Community fund holds the protocol treasury and has control over the protocol owned liquidity.
    It is controlled by the governance contract and serves to grow its holdings and give grants to proposals.
*/

pub(crate) type CommunityFundResult = Result<Response, CommunityFundError>;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let whale_token_addr = deps.api.addr_validate(&msg.whale_token_addr)?;

    let state = State { whale_token_addr };

    STATE.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> CommunityFundResult {
    match msg {
        ExecuteMsg::Spend { recipient, amount } => commands::spend_whale(
            deps.as_ref(),
            info,
            recipient,
            _env.contract.address,
            amount,
        ),
        ExecuteMsg::Burn { amount } => {
            commands::burn_whale(deps.as_ref(), info, _env.contract.address, amount)
        }
        ExecuteMsg::SetAdmin { admin } => commands::set_admin(deps, info, admin),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => Ok(to_binary(&ADMIN.query_admin(deps)?)?),
        QueryMsg::Config {} => query_config(deps),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    to_binary(&ConfigResponse {
        token_addr: state.whale_token_addr,
    })
}
