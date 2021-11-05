#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{Config, State, ALLOCATIONS, CONFIG, STATE};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Api, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use white_whale::vesting::{
    AllocationInfo, AllocationResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    ReceiveMsg, StateResponse, Schedule, SimulateWithdrawResponse,
};

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
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            whale_token: deps.api.addr_validate(&msg.whale_token)?,
            default_unlock_schedule: msg.default_unlock_schedule,
        },
    )?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => handle_receive_cw20(deps, env, info, cw20_msg),
        ExecuteMsg::Withdraw {} => handle_withdraw(deps, env, info),
        ExecuteMsg::TransferOwnership { new_owner } => {
            handle_transfer_ownership(deps, env, info, new_owner)
        }
    }
}

fn handle_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        ReceiveMsg::CreateAllocations { allocations } => handle_create_allocations(
            deps,
            env,
            info.clone(),
            cw20_msg.sender,
            info.sender,
            cw20_msg.amount,
            allocations,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::Allocation { account } => to_binary(&query_allocation(deps, env, account)?),
        QueryMsg::AllAllocations { limit, start_after } => {
            to_binary(&query_all_allocations(deps, env, limit, start_after)?)
        }
        QueryMsg::SimulateWithdraw { account, timestamp } => {
            to_binary(&query_simulate_withdraw(deps, env, account, timestamp)?)
        }
    }
}

//----------------------------------------------------------------------------------------
// Execute Points
//----------------------------------------------------------------------------------------

fn handle_transfer_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    config.owner = deps.api.addr_validate(&new_owner)?;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}



fn handle_create_allocations(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    creator: String,
    deposit_token: Addr,
    deposit_amount: Uint128,
    allocations: Vec<(String, AllocationInfo)>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can create allocations
    if deps.api.addr_validate(&creator)? != config.owner {
        return Err(StdError::generic_err("Only owner can create allocations"));
    }

    // CHECK :: Only WHALE Token can be  can be deposited
    if deposit_token != config.whale_token {
        return Err(StdError::generic_err("Only WHALE token can be deposited"));
    }

    // CHECK :: Number of WHALE Tokens sent need to be equal to the sum of newly vested balances 
    if deposit_amount != allocations.iter().map(|params| params.1.total_amount).sum() {
        return Err(StdError::generic_err("WHALE deposit amount mismatch"));
    }

    for allocation in allocations {
        let (user_unchecked, params) = allocation;

        let user = deps.api.addr_validate(&user_unchecked)?;

        match ALLOCATIONS.load(deps.storage, &user) {
            Ok(..) => {
                return Err(StdError::generic_err("Allocation already exists for user"));
            }
            Err(..) => {
                ALLOCATIONS.save(deps.storage, &user, &params)?;
            }
        }
    }

    Ok(Response::default())
}





fn handle_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let mut allocation = ALLOCATIONS.load(deps.storage, &info.sender)?;

    // Check :: Is valid request ?
    if allocation.total_amount == Uint128::zero()
        || allocation.total_amount == allocation.withdrawn_amount
    {
        return Err(StdError::generic_err("No unlocked WHALE to be withdrawn"));
    }

    let withdrawable_amount = compute_withdraw_amounts(
        env.block.time.seconds(),
        &allocation,
        config.default_unlock_schedule,
    );

    // Check :: Is valid request ?
    if withdrawable_amount == Uint128::zero() {
        return Err(StdError::generic_err("No unlocked WHALE to be withdrawn"));
    }

    // Init Response
    let mut response = Response::new().add_attribute("action", "withdraw");

    // UPDATE :: state & allocation
    allocation.withdrawn_amount += withdrawable_amount;
    state.remaining_whale_tokens -= withdrawable_amount;

    // SAVE :: state & allocation
    STATE.save(deps.storage, &state)?;
    ALLOCATIONS.save(deps.storage, &info.sender, &allocation)?;

    let mut msgs: Vec<WasmMsg> = vec![];

    response = response
        .add_message(WasmMsg::Execute {
            contract_addr: config.whale_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: withdrawable_amount,
            })?,
            funds: vec![],
        })
        .add_attribute("user", info.sender.to_string())
        .add_attribute("withdrawn_amount", withdrawable_amount.to_string());

    Ok(response)
}



pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        whale_token: config.whale_token.to_string(),
        default_unlock_schedule: config.default_unlock_schedule,
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        total_whale_deposited: state.total_whale_deposited,
        remaining_whale_tokens: state.remaining_whale_tokens,
    })
}

fn query_allocation(deps: Deps, _env: Env, account: String) -> StdResult<AllocationResponse> {
    let user_address = deps.api.addr_validate(&account)?;
    let allocation_info = ALLOCATIONS.load(deps.storage, &user_address)?;

    Ok(AllocationResponse {
        total_amount: allocation_info.total_amount,
        withdrawn_amount:  allocation_info.withdrawn_amount,
        vest_schedule: allocation_info.vest_schedule,
    })
}


fn query_simulate_withdraw(
    deps: Deps,
    env: Env,
    account: String,
) -> StdResult<SimulateWithdrawResponse> {
    let user_address = deps.api.addr_validate(&account)?;
    let allocation_info = ALLOCATIONS.load(deps.storage, &user_address)?;

    let config = CONFIG.load(deps.storage)?;
    let mut status = STATUS.load(deps.storage, &account_checked)?;

    Ok(helpers::compute_withdraw_amounts(
        env.block.time.seconds(),
        &params,
        &mut status,
        config.default_unlock_schedule,
    ))
}



pub fn query_vesting_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<VestingAccountsResponse> {
    let vesting_infos = if let Some(start_after) = start_after {
        read_vesting_infos(
            deps.storage,
            Some(deps.api.addr_canonicalize(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        read_vesting_infos(deps.storage, None, limit, order_by)?
    };

    let vesting_account_responses: StdResult<Vec<VestingAccountResponse>> = vesting_infos
        .iter()
        .map(|vesting_account| {
            Ok(VestingAccountResponse {
                address: deps.api.addr_humanize(&vesting_account.0)?.to_string(),
                info: vesting_account.1.clone(),
            })
        })
        .collect();

    Ok(VestingAccountsResponse {
        vesting_accounts: vesting_account_responses?,
    })
}
