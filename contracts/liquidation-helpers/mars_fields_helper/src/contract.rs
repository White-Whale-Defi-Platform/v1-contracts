#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, StdError, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg
};

use whitewhale_periphery::mars_fields_helper::{
    InstantiateMsg, UpdateConfigMsg, ExecuteMsg, QueryMsg, CallbackMsg,
    ConfigResponse, StateResponse, MartianFieldsLiquidationMsg
};
use whitewhale_periphery::helper::{
    build_send_native_asset_msg, option_string_to_addr,
    query_balance
};
use whitewhale_periphery::tax::{
    compute_tax
};
use crate::state::{ Config, State, CONFIG, STATE};
use cosmwasm_bignumber::{Uint256};





#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    let config = Config {
            owner:  deps.api.addr_validate(&msg.owner)?,
            controller_strategy: deps.api.addr_validate(&msg.controller_strategy)?,
            fields_addresses: vec![],
            stable_denom: msg.stable_denom,
    };

    let state = State {
            total_liquidations: 0u64,
            total_ust_profit: Uint256::zero(),
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { new_config } => handle_update_config(deps, info, new_config),
        ExecuteMsg::AddFieldsStrategy { fields_strat_addr } => handle_add_fields_strategy(deps, info, fields_strat_addr),
        ExecuteMsg::LiquidateFieldsPosition { user_address, fields_strat_addr  } => handle_liquidate_fields_position(deps, _env, info, user_address, fields_strat_addr ),
        ExecuteMsg::Callback(msg) => _handle_callback(deps, _env, info, msg)
    }
}



fn _handle_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CallbackMsg,
) -> StdResult<Response> {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(StdError::generic_err(
            "callbacks cannot be invoked externally",
        ));
    }
    match msg {
        CallbackMsg::AfterLiquidationCallback { } => after_liquidation_callback(
            deps,
            env,
        ),
    }
}




#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
    }
}




//----------------------------------------------------------------------------------------
// EXECUTE FUNCTION HANDLERS
//----------------------------------------------------------------------------------------


/// @dev Admin function to update Configuration parameters
/// @param new_config : Same as UpdateConfigMsg struct
pub fn handle_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_config: UpdateConfigMsg,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // UPDATE :: ADDRESSES IF PROVIDED
    config.owner = option_string_to_addr(deps.api, new_config.owner, config.owner)?;
    config.controller_strategy = option_string_to_addr(
        deps.api,
        new_config.controller_strategy,
        config.controller_strategy,
    )?;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "fields_helper::ExecuteMsg::UpdateConfig"))
}


/// @dev Admin function to add a new Fields strategy address
/// @param new_asset : 
pub fn handle_add_fields_strategy(
    deps: DepsMut,
    info: MessageInfo,
    new_fields_strat_addr: String
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    let new_fields_strat_address = deps.api.addr_validate( &new_fields_strat_addr )?;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner.clone() {
        return Err(StdError::generic_err("Unauthorized"));
    }

    for fields_strat_address in config.fields_addresses.iter() {
        if fields_strat_address.clone().to_string() == new_fields_strat_address {
            return Err(StdError::generic_err("Already Supported"));
        }
    }

    config.fields_addresses.push(  new_fields_strat_address.to_string() );

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "fields_helper::ExecuteMsg::AddAsset"))
}




/// @dev 
/// @param  : 
pub fn handle_liquidate_fields_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
    fields_strat_addr: String
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // CHECK :: Only controller_strategy can call this function
    if info.sender != config.controller_strategy {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // CHECK :: Fields strategy address supported or not ?
    for fields_strat_address in config.fields_addresses.iter() {
        if fields_strat_address.to_string() == fields_strat_addr.clone().to_string() {
            return Err(StdError::generic_err("Already Supported"));
        }
    }

    // COSMOS MSGS ::
    // 1. Add LiquidatePosition Msg
    // 2. Add 'AfterLiquidationCallback' Callback Msg  

    let mut cosmos_msgs = vec![];

    // LiquidatePosition Msg
    cosmos_msgs.push( build_liquidate_fields_position_msg(fields_strat_addr, user_address)? );

    // Callback Cosmos Msg 
    let after_masset_buy_callback_msg = CallbackMsg::AfterLiquidationCallback {
    }.to_cosmos_msg(&env.contract.address)?;
    cosmos_msgs.push(after_masset_buy_callback_msg);

    Ok(Response::new().add_messages(cosmos_msgs)
    .add_attribute("action", "fields_helper::ExecuteMsg::LiquidatePosition"))
}



//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

pub fn after_liquidation_callback(
    deps: DepsMut,
    env: Env
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Query UST Balance. Liquidation should be within the { max_loss, +ve } bound
    let cur_ust_balance = query_balance(&deps.querier, env.contract.address, config.stable_denom.clone())?;
    let tax = compute_tax(deps.as_ref(), &Coin { denom: config.stable_denom.clone(), amount: cur_ust_balance, } )?;

    state.total_liquidations += 1u64;
    state.total_ust_profit += Uint256::from(cur_ust_balance - tax);

    // COSMOS MSGS :: 
    // 1. Send UST Back to the UST arb strategy
    // 2. Update Indexes and deposit UST Back into Anchor
    let send_native_asset_msg = build_send_native_asset_msg( deps.as_ref(), config.controller_strategy.clone(), &config.stable_denom, cur_ust_balance.into() )?;
    // TO DO ::: ADD UPDATE INDEXES MSG WHICH UPDATES INDEXES AND DEPOSITS UST INTO ANCHOR (IN THE STRATEGY CONTROLLER CONTRACT)
    // let update_indexes_and_deposit_to_anchor_msg = 

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_messages(vec![send_native_asset_msg]).
    add_attribute("action", "fields_helper::CallbackMsg::AfterLiquidationCallback"))
}



//-----------------------------------------------------------
// QUERY HANDLERS
//-----------------------------------------------------------


/// @dev Returns the contract's configuration
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        controller_strategy: config.controller_strategy.to_string(),
        fields_addresses: config.fields_addresses,
        stable_denom: config.stable_denom,
    })
}


/// @dev Returns the contract's state
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(StateResponse {
        total_liquidations: state.total_liquidations,
        total_ust_profit: state.total_ust_profit
    })
}




//-----------------------------------------------------------
// HELPER FUNCTIONS :: COSMOS MSGs
//-----------------------------------------------------------


fn build_liquidate_fields_position_msg(
    fields_addr: String,
    user_addr: String,
) -> StdResult<CosmosMsg> {

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: fields_addr,
        funds: vec![],
        msg: to_binary(&MartianFieldsLiquidationMsg::Liquidate {
            user: user_addr
        })?,
    }))

}
