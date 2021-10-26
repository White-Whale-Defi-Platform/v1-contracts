#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, StdError, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg, QuerierWrapper, QueryRequest, WasmQuery, Uint128, Decimal
};

use whitewhale_periphery::mirror_helper::{
    InstantiateMsg, UpdateConfigMsg, ExecuteMsg, QueryMsg,
    CallbackMsg, ConfigResponse, StateResponse,
    MAssetInfo,
};
use whitewhale_periphery::helper::{
    build_send_native_asset_msg, option_string_to_addr,
    get_denom_amount_from_coins, query_balance, query_token_balance
};
use whitewhale_periphery::tax::{
    compute_tax
};
use whitewhale_periphery::asset::{
    AssetInfo };
use crate::state::{ Config, State, CONFIG, STATE};
use cosmwasm_bignumber::{Decimal256, Uint256};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    let config = Config {
            owner:  deps.api.addr_validate(&msg.owner)?,
            controller_strategy: deps.api.addr_validate(&msg.controller_strategy)?,
            mirror_mint_contract: deps.api.addr_validate(&msg.mirror_mint_contract)?,
            stable_denom: msg.stable_denom,
            massets_supported: vec![]
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
        ExecuteMsg::AddMasset { new_masset_info, pair_address } => handle_add_masset(deps, info, new_masset_info, pair_address),
        ExecuteMsg::LiquidateMirrorPosition { position_idx, max_loss_amount  } => handle_liquidate_mirror_position(deps, _env, info, position_idx, max_loss_amount),
        ExecuteMsg::Callback(msg) => _handle_callback(deps, _env, info, msg),
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
        CallbackMsg::AftermAssetBuyCallback {
            position_idx,
            minted_masset,    
            minted_pair_addr,        
            collateral_masset,
            collateral_pair_addr,
            ust_amount,
            max_loss_amount,
        } => after_masset_buy_callback(
            deps,
            env,
            position_idx,
            minted_masset,
            minted_pair_addr,
            collateral_masset,
            collateral_pair_addr,
            ust_amount,
            max_loss_amount
        ),
        CallbackMsg::AfterLiquidationCallback {
            minted_masset,
            minted_pair_addr,
            collateral_masset,
            collateral_pair_addr,
            ust_amount,
            max_loss_amount,            
        } => after_liquidation_callback(
            deps,
            env,
            minted_masset,
            minted_pair_addr,
            collateral_masset,
            collateral_pair_addr,
            ust_amount,
            max_loss_amount,    
        ),
        CallbackMsg::AftermAssetsSellCallback {
            ust_amount,
            max_loss_amount,    
        } => after_massets_sell_callback(
            deps,
            env,
            ust_amount,
            max_loss_amount,    
        )        
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
    config.mirror_mint_contract = option_string_to_addr(
        deps.api,
        new_config.mirror_mint_contract,
        config.mirror_mint_contract,
    )?;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "mirror_helper::ExecuteMsg::UpdateConfig"))
}



/// @dev Admin function to add a new mAsset
/// @param new_masset : 
pub fn handle_add_masset(
    deps: DepsMut,
    info: MessageInfo,
    new_masset: AssetInfo,
    pair_address: String,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let mut masset_config = mirror_protocol::mint::AssetConfigResponse {
        token: "".to_string(),
        auction_discount: Decimal::zero(),
        min_collateral_ratio: Decimal::zero(),
        end_price: None,
        ipo_params: None
    };

    match new_masset.clone() {
        AssetInfo::Token {contract_addr} => {
            masset_config = query_masset_config(&deps.querier,config.mirror_mint_contract.to_string() , contract_addr.to_string() )?;     
            if masset_config.token != contract_addr {
                return Err(StdError::generic_err("Invalid asset address. Not supported by Mirror"));
            }        
        },
        AssetInfo::NativeToken {denom} => { }        
    }

    for masset_ in config.massets_supported.iter() {
        if masset_.asset_token == new_masset {
            return Err(StdError::generic_err("Already Supported"));
        }
    }

    let new_masset_info = MAssetInfo {
        asset_token: new_masset,
        pair_address: deps.api.addr_validate( &pair_address)?,
        auction_discount: masset_config.auction_discount.into() , 
        min_collateral_ratio: masset_config.min_collateral_ratio.into() 
    };

    config.massets_supported.push(new_masset_info);

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "mirror_helper::ExecuteMsg::AddCollateral"))
}


/// @dev 
/// @param  : 
pub fn handle_liquidate_mirror_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    position_idx: Uint128,
    max_loss_amount: Uint256,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // CHECK :: Only controller_strategy can call this function
    if info.sender != config.controller_strategy {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // get UST sent for the liquidation
    let ust_amount = get_denom_amount_from_coins(&info.funds, &config.stable_denom);

    // Query Position's Info    
    let position_info = query_position( &deps.querier, config.mirror_mint_contract.to_string(), position_idx )?;

    // Get mAsset minted against the position which needs to be bought and returned
    let minted_masset  : AssetInfo;
    let minted_masset_addr : Addr;
    match position_info.asset.info {
        terraswap::asset::AssetInfo::Token { contract_addr } => { 
            minted_masset = AssetInfo::Token { contract_addr: deps.api.addr_validate( &contract_addr)?  }; 
            minted_masset_addr = deps.api.addr_validate( &contract_addr)? ;
        },
        _ => { return Err(StdError::generic_err("Invalid Position query response. mAsset can only be a cw20 token") ); }
    }

    // Get collateral asset and it's type which will be retuned
    let collateral_masset : AssetInfo;
    match position_info.collateral.info {
        terraswap::asset::AssetInfo::Token { contract_addr } => { 
            collateral_masset = AssetInfo::Token { contract_addr: deps.api.addr_validate( &contract_addr)?  }; 
        },
        terraswap::asset::AssetInfo::NativeToken { denom } => { 
            collateral_masset = AssetInfo::NativeToken { denom: denom }  ; 
        },
        _ => { return Err(StdError::generic_err("Invalid Position query response. Invalid Collateral") ); }
    }

    // get minted & collateral mAsset's asssociated Terraswap Pair addresses
    let mut minted_pair_address = "".to_string();
    let mut collateral_pair_address = "".to_string();
    for masset_ in config.massets_supported.iter() {
        if masset_.asset_token == minted_masset {
            minted_pair_address = masset_.pair_address.to_string();
        }
        if masset_.asset_token == collateral_masset {
            collateral_pair_address = masset_.pair_address.to_string();
        }        
    }

    let mut cosmos_msgs = vec![];

    // COSMOS MSGS ::
    // 1. Add Buy mAsset from Terraswap Msg
    // 2. Add 'AftermAssetBuyCallback' Callback Msg  
    
    cosmos_msgs.push( build_buy_cw20_with_native_terraswap_msg( deps.as_ref(), minted_pair_address.clone(), config.stable_denom, ust_amount.into() )? );
 
    // Callback Cosmos Msg :: To send liquidation tx after the mAsset to be returned has been bought on Terraswap
    let after_masset_buy_callback_msg = CallbackMsg::AftermAssetBuyCallback {
        position_idx: position_idx,
        minted_masset: minted_masset_addr.clone(),
        minted_pair_addr: minted_pair_address,
        collateral_masset: collateral_masset,
        collateral_pair_addr: collateral_pair_address,
        ust_amount: ust_amount.clone(),
        max_loss_amount: max_loss_amount
    }
    .to_cosmos_msg(&env.contract.address)?;
    cosmos_msgs.push(after_masset_buy_callback_msg);

    Ok(Response::new().add_messages(cosmos_msgs)
    .add_attribute("action", "mirror_helper::ExecuteMsg::LiquidatePosition"))
}

//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

pub fn after_masset_buy_callback(
    deps: DepsMut,
    env: Env,
    position_idx: Uint128,
    minted_masset: Addr,
    minted_pair_addr: String,
    collateral_masset: AssetInfo,
    collateral_pair_addr: String,
    ust_amount: Uint256,
    max_loss_amount: Uint256
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // Query how much minted mAsset (to be returned) was received against UST from terraswap
    let minted_massets_balance = query_token_balance(&deps.querier, minted_masset.clone(), env.contract.address.clone() )?;

    // COSMOS MSGS ::
    // 1. Add LiquidatePosition Msg
    // 2. Add 'AfterLiquidationCallback' Callback Msg  
    let mut cosmos_msgs = vec![];
    cosmos_msgs.push( build_liquidate_position( config.mirror_mint_contract.to_string(), position_idx )? );
    let after_liquidation_callback_msg = CallbackMsg::AfterLiquidationCallback {
        minted_masset: minted_masset,
        minted_pair_addr: minted_pair_addr,
        collateral_masset: collateral_masset,
        collateral_pair_addr: collateral_pair_addr,
        ust_amount: ust_amount,
        max_loss_amount: max_loss_amount,    
    }.to_cosmos_msg(&env.contract.address)?;
    cosmos_msgs.push( after_liquidation_callback_msg );

    Ok(Response::new().add_messages(cosmos_msgs).add_attribute("action", "mirror_helper::CallbackMsg::AftermAssetBuyCallback"))
}


pub fn after_liquidation_callback(
    deps: DepsMut,
    env: Env,
    minted_masset: Addr,
    minted_pair_addr: String,
    collateral_masset: AssetInfo,
    collateral_pair_addr: String,
    ust_amount: Uint256,
    max_loss_amount: Uint256
) -> StdResult<Response> {

    let mut cosmos_msgs = vec![];

    // Query minted mAsset & returned collateral asset balance
    // COSMOS MSGS ::
    // 1. Sell remaining bought mAsset (if any) for UST 
    // 2. Sell received collateral mAssetfor UST
    // 3. Add 'AftermAssetsSellCallback' Callback Msg  


    let minted_massets_balance = query_token_balance(&deps.querier, minted_masset.clone(), env.contract.address.clone() )?;
    if minted_massets_balance > Uint128::zero() {
        cosmos_msgs.push( build_sell_cw20_for_native_terraswap_msg(minted_pair_addr, minted_masset.to_string() , minted_massets_balance)? );
    }

    let collateral_masset_balance : Uint256;
    match collateral_masset {
        AssetInfo::Token { contract_addr }  => {
            collateral_masset_balance = query_token_balance(&deps.querier, contract_addr.clone(), env.contract.address.clone() )?.into();
            cosmos_msgs.push( build_sell_cw20_for_native_terraswap_msg(collateral_pair_addr, contract_addr.clone().to_string() , collateral_masset_balance.into() )? );
        },
        AssetInfo::NativeToken { denom }  => {
            if denom.clone() != "uusd".to_string() {
                collateral_masset_balance = query_balance(&deps.querier, env.contract.address.clone(),  denom.clone() )?.into();
                cosmos_msgs.push( build_sell_native_for_ust_terraswap_msg(deps.as_ref(), collateral_pair_addr , denom.clone(), collateral_masset_balance.into() )? );
            }
        }
    }

    let after_masset_sell_callback_msg = CallbackMsg::AftermAssetsSellCallback {
        ust_amount: ust_amount,
        max_loss_amount: max_loss_amount,    
    }.to_cosmos_msg(&env.contract.address)?;
    cosmos_msgs.push( after_masset_sell_callback_msg );

    Ok(Response::new().add_messages(cosmos_msgs).add_attribute("action", "mirror_helper::CallbackMsg::AfterLiquidationCallback"))
}



pub fn after_massets_sell_callback(
    deps: DepsMut,
    env: Env,
    ust_amount: Uint256,
    max_loss_amount: Uint256
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Query UST Balance. Liquidation should be within the { max_loss, +ve } bound
    let cur_ust_balance = query_balance(&deps.querier, env.contract.address, config.stable_denom.clone())?;
    let tax = compute_tax(deps.as_ref(), &Coin { denom: config.stable_denom.clone(), amount: cur_ust_balance, } )?;
    let min_ust_balance = ust_amount - max_loss_amount;
    let computed_ust_balance = Uint256::from(cur_ust_balance - tax) ;

    if min_ust_balance >= computed_ust_balance {
        return Err(StdError::generic_err(format!("UST received post liquidation = {} is less than minimum needed UST balance = {}",computed_ust_balance, min_ust_balance )));
    }

    state.total_liquidations += 1u64;
    state.total_ust_profit += computed_ust_balance - min_ust_balance  ;

    // COSMOS MSGS :: 
    // 1. Send UST Back to the UST arb strategy
    // 2. Update Indexes and deposit UST Back into Anchor
    let send_native_asset_msg = build_send_native_asset_msg( deps.as_ref(), config.controller_strategy.clone(), &config.stable_denom, cur_ust_balance.into() )?;
    // TO DO ::: ADD UPDATE INDEXES MSG WHICH UPDATES INDEXES AND DEPOSITS UST INTO ANCHOR (IN THE STRATEGY CONTROLLER CONTRACT)
    // let update_indexes_and_deposit_to_anchor_msg = 

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_messages(vec![send_native_asset_msg]).
    add_attribute("action", "mirror_helper::CallbackMsg::AftermAssetsSellCallback"))
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
        mirror_mint_contract: config.mirror_mint_contract.to_string(),
        stable_denom: config.stable_denom,
        massets_supported: config.massets_supported
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
// HELPER FUNCTIONS :: QUERY MSGs
//-----------------------------------------------------------











/// @dev query position info from the mirror's mint contract
pub fn query_position(
    querier: &QuerierWrapper,
    mint_addr: String,
    position_idx: Uint128
) -> StdResult< mirror_protocol::mint::PositionResponse > {
    let res: mirror_protocol::mint::PositionResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: mint_addr,
            msg: to_binary(&mirror_protocol::mint::QueryMsg::Position {
                position_idx: position_idx
            })?,
        }))?;
    Ok(res)
}


/// @dev query mAsset's config info from the mirror's mint contract
pub fn query_masset_config(
    querier: &QuerierWrapper,
    mint_addr: String,
    masset_token_addr: String
) -> StdResult<  mirror_protocol::mint::AssetConfigResponse > {
    let res: mirror_protocol::mint::AssetConfigResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: mint_addr,
            msg: to_binary(&mirror_protocol::mint::QueryMsg::AssetConfig {
                asset_token: masset_token_addr
            })?,
        }))?;
    Ok(res)
}



//-----------------------------------------------------------
// HELPER FUNCTIONS :: COSMOS MSGs
//-----------------------------------------------------------



/// @dev Returns a Cosmos Msg to buy an mAsset (CW20) against UST via terraswap
fn build_buy_cw20_with_native_terraswap_msg(
    deps: Deps,
    pair_address: String,
    denom: String,
    denom_buy_worth: Uint128
) -> StdResult<CosmosMsg> {
    let tax = compute_tax(deps, &Coin { denom: denom.clone(), amount: denom_buy_worth } )?;
    let denom_to_send = denom_buy_worth - tax ;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:pair_address,
        funds: vec![Coin {denom: denom.to_string(), amount: denom_to_send.into() } ],
        msg: to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: terraswap::asset::Asset {
                info: terraswap::asset::AssetInfo::NativeToken { denom: denom },
                amount: denom_to_send
            } ,
            belief_price: None,
            max_spread: None,
            to: None
        })?,
    }))
}

/// @dev Returns a Cosmos Msg to sell an mAsset (CW20)  via terraswap
fn build_sell_cw20_for_native_terraswap_msg(
    pair_address: String,
    masset_to_sell_addr: String,
    amount: Uint128
) -> StdResult<CosmosMsg> {

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:pair_address,
        funds: vec![],
        msg: to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: terraswap::asset::Asset {
                info: terraswap::asset::AssetInfo::Token { contract_addr: masset_to_sell_addr  },
                amount: amount
            } ,
            belief_price: None,
            max_spread: None,
            to: None
        })?,
    }))
}

/// @dev Returns a Cosmos Msg to sell a Native asset for UST  via terraswap
fn build_sell_native_for_ust_terraswap_msg(
    deps: Deps,    
    pair_address: String,
    native_denom: String,
    amount: Uint128
) -> StdResult<CosmosMsg> {
    let tax = compute_tax(deps, &Coin { denom: native_denom.clone(), amount: amount } )?;
    let denom_to_send = amount - tax ;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:pair_address,
        funds: vec![],
        msg: to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: terraswap::asset::Asset {
                info: terraswap::asset::AssetInfo::NativeToken { denom: native_denom  },
                amount: amount
            } ,
            belief_price: None,
            max_spread: None,
            to: None
        })?,
    }))
}





/// @dev Returns Cosmos Msg to liquidate a CDP position on mirror protocol
fn build_liquidate_position(
    mirror_mint_contract: String,
    position_idx: Uint128
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:mirror_mint_contract,
        funds: vec![],
        msg: to_binary(&mirror_protocol::mint::Cw20HookMsg::Auction {
            position_idx: position_idx
        })?,
    }))
}













