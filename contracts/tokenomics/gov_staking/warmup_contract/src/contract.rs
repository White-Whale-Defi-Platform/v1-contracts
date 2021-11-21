use cosmwasm_std::{
    entry_point, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use white_whale::gov_staking::warmup::{ExecuteMsg, InstantiateMsg};
use white_whale::tokenomics::helpers::build_transfer_cw20_token_msg;

use crate::state::{Config, CONFIG};

//----------------------------------------------------------------------------------------
// Entry Points
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        whale_token: deps.api.addr_validate(&msg.whale_token)?,
        staked_whale_token: deps.api.addr_validate(&msg.staked_whale_token)?,
        whale_staking_contract: deps.api.addr_validate(&msg.whale_staking_contract)?,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Retrieve { staker, amount } => {
            execute_retrieve(deps, env, info, staker, amount)
        }
    }
}

//----------------------------------------------------------------------------------------
// Handle Functions
//----------------------------------------------------------------------------------------

pub fn execute_retrieve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    staker: String,
    amount: Uint128,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    if info.sender != config.whale_staking_contract {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let transfer_swhale_msg = build_transfer_cw20_token_msg(
        deps.api.addr_validate(&staker)?,
        config.staked_whale_token.to_string(),
        amount,
    )?;

    Ok(Response::new()
        .add_message(transfer_swhale_msg)
        .add_attributes(vec![
            ("action", "RetrieveFromWarmup"),
            ("staker", staker.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}
