use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use white_whale::tokenomics::lp_emissions::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    StakerInfoResponse, StateResponse,
};

use crate::state::{Config, StakerInfo, State, CONFIG, STAKER_INFO, STATE};

//----------------------------------------------------------------------------------------
// Entry Points
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        whale_token: deps.api.addr_validate(&msg.whale_token)?,
        distribution_schedule: (0, 0, Uint128::zero()),
    };

    CONFIG.save(deps.storage, &config)?;

    STATE.save(
        deps.storage,
        &State {
            last_distributed: env.block.time.seconds(),
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
            leftover: Uint128::zero(),
            reward_rate_per_token: Decimal::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        // ExecuteMsg::UpdateConfig { new_owner } => update_config(deps, env, info, new_owner),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        // ExecuteMsg::Unbond {
        //     amount,
        //     withdraw_pending_reward,
        // } => unbond(deps, env, info, amount, withdraw_pending_reward),
        // ExecuteMsg::Claim {} => try_claim(deps, env, info),
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::Config {} => to_binary(&query_config(deps)?),
//         QueryMsg::State { timestamp } => to_binary(&query_state(deps, _env, timestamp)?),
//         QueryMsg::StakerInfo { staker, timestamp } => {
//             to_binary(&query_staker_info(deps, _env, staker, timestamp)?)
//         }
//         QueryMsg::Timestamp {} => to_binary(&query_timestamp(_env)?),
//     }
// }

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
//     Err(StdError::generic_err("unimplemented"))
// }
//----------------------------------------------------------------------------------------
// Handle Functions
//----------------------------------------------------------------------------------------

/// Only WHALE-UST LP Token can be sent to this contract via the Cw20ReceiveMsg Hook
/// @dev Increases user's staked LP Token balance via the Bond Function
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            // only WHALE token contract can execute this message
            if config.whale_token != info.sender.as_str() {
                return Err(StdError::generic_err("unauthorized"));
            }
            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            bond(deps, env, cw20_sender, cw20_msg.amount.into())
        }
        // Ok(Cw20HookMsg::UpdateRewardSchedule {
        //     period_start,
        //     period_finish,
        //     amount,
        // }) => {
        //     // Only WHALE token contract can execute this message
        //     if config.whale_token != info.sender.as_str() {
        //         return Err(StdError::generic_err(
        //             "Unauthorized : Only WHALE Token is allowed",
        //         ));
        //     }
        //     // Only owner can update the schedule
        //     if config.owner != cw20_msg.sender {
        //         return Err(StdError::generic_err("Only owner can update the schedule"));
        //     }
        //     update_reward_schedule(
        //         deps,
        //         env,
        //         info,
        //         period_start,
        //         period_finish,
        //         amount,
        //         cw20_msg.amount.into(),
        //     )
        // }
        Err(_) => Err(StdError::generic_err("data should be given")),
    }
}

/// @dev Called by receive_cw20(). Increases user's staked WHALE Token balance, Us
/// @params sender_addr : User Address who sent the WHALE Tokens
/// @params amount : Number of WHALE Tokens transferred to the contract
pub fn bond(deps: DepsMut, env: Env, sender_addr: Addr, amount: Uint128) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &sender_addr)?
        .unwrap_or_default();

    // rebase()

    if staker_info.lock {
        return Err(StdError::generic_err("Deposits for account are locked"));
    }

    staker_info.staked_amount += amount;
    // staker_info.gons += IsOHM(config.staked_whale_token).gonsForBalance(_amount);
    staker_info.expiry += state.warmupPeriod;
    staker_info.lock = false;
    STAKER_INFO.save(deps.storage, &sender_addr, &staker_info)?;

    state.total_staked_amount += amount;
    STATE.save(deps.storage, &state)?;

    let transfer_swhale_msg = build_transfer_cw20_token_msg(
        sender_addr.clone(),
        config.staked_whale_token,
        amount ,
    )?

    Ok(Response::new().add_message(transfer_swhale_msg).add_attributes(vec![
        ("action", "Bond"),
        ("user", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

/// @dev Only owner can call this function. Updates the config
/// @params new_owner : New owner address
// pub fn update_config(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     new_owner: String,
// ) -> StdResult<Response> {
//     let mut config = CONFIG.load(deps.storage)?;

//     // ONLY OWNER CAN UPDATE CONFIG
//     if info.sender != config.owner {
//         return Err(StdError::generic_err("Only owner can update configuration"));
//     }

//     // UPDATE :: ADDRESSES IF PROVIDED
//     config.owner = deps.api.addr_validate(&new_owner)?;
//     CONFIG.save(deps.storage, &config)?;

//     Ok(Response::new()
//         .add_attribute("action", "UpdateConfig")
//         .add_attribute("new_owner", new_owner))
// }



pub fn claim(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &msg.sender.clone())?
        .unwrap_or_default();

    if (state.epoch.number >= staker_info.expiry && staker_info.expiry != 0  ) {

        // let claim_amount =  IsOHM(config.staked_whale_token).gonsForBalance( staker_info.gons );
        // delete staker_info[ msg.sender ];
        let transfer_swhale_msg = build_transfer_cw20_token_msg(
            sender_addr.clone(),
            config.staked_whale_token,
            claim_amount ,
        )?
    }
}


pub fn forfeit(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &msg.sender.clone())?
        .unwrap_or_default();


    // let claim_amount =  IsOHM(config.staked_whale_token).gonsForBalance( staker_info.gons );
    // delete staker_info[ msg.sender ];
    let transfer_swhale_msg = build_transfer_cw20_token_msg(
        sender_addr.clone(),
        config.staked_whale_token,
        claim_amount ,
    )?

    let transfer_whale_msg = build_transfer_cw20_token_msg(
        sender_addr.clone(),
        config.whale_token,
        staker_info.staked_amount ,
    )?
}




pub fn toggleDepositLock(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo
) -> StdResult<Response> {
    let mut staker_info = STAKER_INFO
        .load(deps.storage, &msg.sender.clone())?;
    
        staker_info.lock = true;
    
    Response::Default()
}



/// @dev CW20Hook Function callable by S-WHALE tokens. Used to unbond position by returning S-WHALE tokens. Users gets back WHALE tokens
/// @params sender_addr : User Address who sent the S-WHALE Tokens
/// @params amount : Number of S-WHALE Tokens transferred to the contract
pub fn unbond(deps: DepsMut, env: Env, sender_addr: Addr, amount: Uint128, trigger: bool) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    
    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &sender_addr)?
        .unwrap_or_default();

    if trigger {
        // rebase()
    }

    let transfer_swhale_msg = build_transfer_cw20_token_msg(
        sender_addr.clone(),
        config.whale_token,
        amount ,
    )?

    Ok(Response::new().add_message(transfer_swhale_msg).add_attributes(vec![
        ("action", "UnBond"),
        ("user", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}


pub fn rebase(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;    
    let mut state: State = STATE.load(deps.storage)?;

    if state.epoch.endBlock <= env.block.number {
        // IsOHM( sOHM ).rebase( epoch.distribute, epoch.number );
        state.epoch.endBlock = + state.epoch.length;
        state.epoch.number += 1;
    }

    // if ( distributor != address(0) ) {
    //     IDistributor( distributor ).distribute();
    // }

    // let balance = IERC20( OHM ).balanceOf( address(this) ).add( totalBonus );
    // let staked = IsOHM( sOHM ).circulatingSupply();

    if  balance <= staked {
        epoch.distribute = 0;
    } else {
        epoch.distribute = balance.sub( staked );
    }

    Ok(Response::new().add_attributes(vec![
        ("action", "Rebase")
    ]))
}


/// @dev Provide bonus to locked staking contract
pub fn give_lock_bonus(deps: DepsMut, env: Env, amount: Uint128) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;   
    
    if msg.sender != config.locker {
        return Err(StdError::generic_err("Only Locker can execute"));
    }
    
    let mut state: State = STATE.load(deps.storage)?;
    state.total_bonus += amount;

    let transfer_swhale_msg = build_transfer_cw20_token_msg(
        msg.sender.clone().to_string(),
        config.staked_whale_token,
        amount ,
    )?    

    Ok(Response::new().add_message(transfer_swhale_msg).add_attributes(vec![
        ("action", "GiveLockBonus")
    ]))
}



/// @dev CW20Hook ::: Reclaim bonus from locked staking contract
pub fn return_lock_bonus(deps: DepsMut, env: Env, sender: Addr, amount: Uint128) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;   
    
    if sender != config.locker {
        return Err(StdError::generic_err("Only Locker can execute"));
    }
    
    let mut state: State = STATE.load(deps.storage)?;
    state.total_bonus = state.total_bonus - amount;  

    Ok(Response::new().add_message(transfer_swhale_msg).add_attributes(vec![
        ("action", "ReturnLockBonus"), ("amount_returned", amount)
    ]))
}






//----------------------------------------------------------------------------------------
// Query Functions
//----------------------------------------------------------------------------------------

/// @dev Returns the contract's configuration
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        whale_token: config.whale_token.to_string(),
        staking_token: config.staking_token.to_string(),
        distribution_schedule: config.distribution_schedule,
    })
}

/// @dev Returns the contract's simulated state at a certain timestamp
/// /// @param timestamp : Option parameter. Contract's Simulated state is retrieved if the timestamp is provided   
pub fn query_state(deps: Deps, env: Env, timestamp: Option<u64>) -> StdResult<StateResponse> {
    let mut state: State = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    match timestamp {
        Some(timestamp) => {
            compute_reward(
                &config,
                &mut state,
                std::cmp::max(timestamp, env.block.time.seconds()),
            );
        }
        None => {
            compute_reward(&config, &mut state, env.block.time.seconds());
        }
    }

    Ok(StateResponse {
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index: state.global_reward_index,
        leftover: state.leftover,
        reward_rate_per_token: state.reward_rate_per_token,
    })
}

/// @dev Returns the User's simulated state at a certain timestamp
/// @param staker : User address whose state is to be retrieved
/// @param timestamp : Option parameter. User's Simulated state is retrieved if the timestamp is provided   
pub fn query_staker_info(
    deps: Deps,
    env: Env,
    staker: String,
    timestamp: Option<u64>,
) -> StdResult<StakerInfoResponse> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &deps.api.addr_validate(&staker)?)?
        .unwrap_or_default();

    match timestamp {
        Some(timestamp) => {
            compute_reward(
                &config,
                &mut state,
                std::cmp::max(timestamp, env.block.time.seconds()),
            );
        }
        None => {
            compute_reward(&config, &mut state, env.block.time.seconds());
        }
    }

    compute_staker_reward(&state, &mut staker_info)?;

    Ok(StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}

/// @dev Returns the current timestamp
pub fn query_timestamp(env: Env) -> StdResult<u64> {
    Ok(env.block.time.seconds())
}

//----------------------------------------------------------------------------------------
// Helper Functions
//----------------------------------------------------------------------------------------

/// @dev Increases total LP shares and user's staked LP shares by `amount`
fn increase_bond_amount(state: &mut State, staker_info: &mut StakerInfo, amount: Uint128) {
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
}

/// @dev Decreases total LP shares and user's staked LP shares by `amount`
fn decrease_bond_amount(state: &mut State, staker_info: &mut StakerInfo, amount: Uint128) {
    state.total_bond_amount = state.total_bond_amount - amount;
    staker_info.bond_amount = staker_info.bond_amount - amount;
}

/// @dev Updates State's leftover and reward_rate_per_token params
fn compute_state_extra(config: &Config, state: &mut State, timestamp: u64) {
    let s = config.distribution_schedule;

    // not started yet
    if timestamp <= s.0 {
        state.leftover = s.2;
        state.reward_rate_per_token = Decimal::zero();
    }
    // already finished
    else if timestamp >= s.1 {
        state.leftover = Uint128::zero();
        state.reward_rate_per_token = Decimal::zero();
    }
    // s.0 < timestamp < s.1
    else {
        let duration = s.1 - s.0;
        let distribution_rate: Decimal = Decimal::from_ratio(s.2, duration);
        let time_left = s.1 - timestamp;
        state.leftover = distribution_rate * Uint128::from(time_left as u128);
        if state.total_bond_amount.is_zero() {
            state.reward_rate_per_token = Decimal::zero();
        } else {
            let denom = Uint128::from(10u128.pow(config.staking_token_decimals as u32));
            state.reward_rate_per_token =
                Decimal::from_ratio(distribution_rate * denom, state.total_bond_amount);
        }
    }
}

// compute distributed rewards and update global reward index
fn compute_reward(config: &Config, state: &mut State, timestamp: u64) {
    compute_state_extra(config, state, timestamp);

    if state.total_bond_amount.is_zero() {
        state.last_distributed = timestamp;
        return;
    }

    let mut distributed_amount: Uint128 = Uint128::zero();
    let s = config.distribution_schedule;
    if s.0 < timestamp && s.1 > state.last_distributed {
        let time_passed =
            std::cmp::min(s.1, timestamp) - std::cmp::max(s.0, state.last_distributed);
        let duration = s.1 - s.0;
        let distribution_rate: Decimal = Decimal::from_ratio(s.2, duration);
        distributed_amount += distribution_rate * Uint128::from(time_passed as u128);
    }

    state.last_distributed = timestamp;
    state.global_reward_index = state.global_reward_index
        + Decimal::from_ratio(distributed_amount, state.total_bond_amount);
}

/// @dev Computes user's accrued rewards
fn compute_staker_reward(state: &State, staker_info: &mut StakerInfo) -> StdResult<()> {
    let pending_reward = (staker_info.bond_amount * state.global_reward_index)
        - (staker_info.bond_amount * staker_info.reward_index);
    staker_info.reward_index = state.global_reward_index;
    staker_info.pending_reward += pending_reward;
    Ok(())
}

/// @dev Helper function to build `CosmosMsg` to send cw20 tokens to a recepient address
fn build_transfer_cw20_token_msg(
    recipient: Addr,
    token_contract_address: Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_contract_address.into(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount: amount.into(),
        })?,
        funds: vec![],
    }))
}
