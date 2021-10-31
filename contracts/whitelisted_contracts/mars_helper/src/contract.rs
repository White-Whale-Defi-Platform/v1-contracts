#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, StdError, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg, QuerierWrapper, QueryRequest, WasmQuery
};

use whitewhale_liquidation_helpers::mars_helper::{
    InstantiateMsg, UpdateConfigMsg, ExecuteMsg, QueryMsg, CallbackMsg,
    ConfigResponse, StateResponse, MarsLiquidationMsg, MarsQueryMsg, RedBankMarketQueryResponse,
    MarsAsset, RedBankAssetsInfo
};
use whitewhale_liquidation_helpers::helper::{
    build_send_cw20_token_msg, build_send_native_asset_msg, option_string_to_addr,
    get_denom_amount_from_coins, query_balance, query_token_balance
};
use whitewhale_liquidation_helpers::nft_minter::ExecuteMsg as MinterExecuteMsg;
use whitewhale_liquidation_helpers::tax::{
    compute_tax, deduct_tax
};
use whitewhale_liquidation_helpers::astroport_helper::{
    trade_cw20_for_native_on_astroport, trade_native_for_cw20_on_astroport, trade_native_for_native_on_astroport
};
use whitewhale_liquidation_helpers::flashloan_helper::build_flash_loan_msg;

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
            ust_vault_address: deps.api.addr_validate(&msg.ust_vault_address)?,
            red_bank_addr: deps.api.addr_validate(&msg.red_bank_addr)?,
            astroport_router: deps.api.addr_validate(&msg.astroport_router)?,
            stable_denom: msg.stable_denom,
            assets_supported: vec![]
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
        ExecuteMsg::AddAsset { new_asset } => handle_add_asset(deps, info, new_asset),
        ExecuteMsg::LiquidateRedBankUser { user_address,ust_to_borrow, debt_asset, collateral_asset, max_loss_amount  } => handle_liquidate_red_bank_user(deps, _env, info, user_address, ust_to_borrow, debt_asset, collateral_asset, max_loss_amount ),
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
        CallbackMsg::InitiateLiquidationCallback {
            user_address, 
            debt_asset, 
            collateral_asset, 
            max_loss_amount 
        } => initiate_liquidation_callback(
            deps,
            env,
            info,
            user_address, 
            debt_asset, 
            collateral_asset, 
            max_loss_amount 
        ),
        CallbackMsg::AfterDebtAssetBuyCallback {
            user_address, 
            debt_asset, 
            collateral_asset, 
            ust_amount,
            max_loss_amount 
        } => after_debt_asset_callback(
            deps,
            env,
            user_address, 
            debt_asset, 
            collateral_asset, 
            ust_amount,
            max_loss_amount 
        ),
        CallbackMsg::AfterLiquidationCallback {
            debt_asset, 
            collateral_asset, 
            ust_amount,
            max_loss_amount           
        } => after_liquidation_callback(
            deps,
            env,
            debt_asset, 
            collateral_asset, 
            ust_amount,
            max_loss_amount    
        ),
        CallbackMsg::AfterAssetsSellCallback {
            ust_amount,
            max_loss_amount,    
        } => after_assets_sell_callback(
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
    config.ust_vault_address = option_string_to_addr(
        deps.api,
        new_config.ust_vault_address,
        config.ust_vault_address,
    )?;
    config.red_bank_addr = option_string_to_addr(
        deps.api,
        new_config.red_bank_addr,
        config.red_bank_addr,
    )?;
    config.astroport_router = option_string_to_addr(
        deps.api,
        new_config.astroport_router,
        config.astroport_router,
    )?;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "redbank_helper::ExecuteMsg::UpdateConfig"))
}


/// @dev Admin function to add a new asset supported by Red Bank
/// @param new_asset : 
pub fn handle_add_asset(
    deps: DepsMut,
    info: MessageInfo,
    new_asset: MarsAsset,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    for masset_ in config.assets_supported.iter() {
        if masset_.asset_info == new_asset.clone() {
            return Err(StdError::generic_err("Already Supported"));
        }
    }

    let market_res = query_market( &deps.querier, config.red_bank_addr.to_string(), new_asset.clone())?;
    if !market_res.active {
        return Err(StdError::generic_err("maAsset not currently active with Red Bank"));
    }

    config.assets_supported.push(RedBankAssetsInfo { 
        asset_info: new_asset ,
        ma_token_address: market_res.ma_token_address  ,
        max_loan_to_value: market_res.max_loan_to_value.into() ,
        liquidation_threshold: market_res.liquidation_threshold.into() ,
        liquidation_bonus: market_res.liquidation_bonus.into() 

    });

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "redbank_helper::ExecuteMsg::AddAsset"))
}



/// @dev 
/// @param  : 
pub fn handle_liquidate_red_bank_user(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
    ust_to_borrow: Uint256,
    debt_asset: MarsAsset,
    collateral_asset: MarsAsset,
    max_loss_amount: Uint256,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let callback_binary = to_binary(&CallbackMsg::InitiateLiquidationCallback {
                                        user_address: user_address.clone(),
                                        debt_asset: debt_asset,
                                        collateral_asset: collateral_asset,
                                        max_loss_amount: max_loss_amount.into(),
                                    }.to_cosmos_msg(&env.contract.address)?)?;

    let flash_loan_msg = build_flash_loan_msg( config.ust_vault_address.to_string(),
                                config.stable_denom,
                                ust_to_borrow,
                                callback_binary )?;
    
    Ok(Response::new()
    .add_message(flash_loan_msg)
    .add_attribute("action", "redbank_helper::ExecuteMsg::LiquidatePosition")
    .add_attribute("user_address", user_address )
    .add_attribute("loan_asked", ust_to_borrow.to_string() ))
                        
}


//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

pub fn initiate_liquidation_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
    debt_asset: MarsAsset,
    collateral_asset: MarsAsset,
    max_loss_amount: Uint256
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // get UST sent for the liquidation
    let ust_amount = get_denom_amount_from_coins(&info.funds, &config.stable_denom);

    // COSMOS MSGS ::
    // 1. Add Buy debt asset from Astroport Msg
    // 2. Add 'AfterDebtAssetBuyCallback' Callback Msg  

    // Init response
    let mut response = Response::new().add_attribute("Action", "Mars_Liquidation");

    // Add the specific Cosmos Msg to buy the debt asset to be paid back to the Response
    match debt_asset.clone() {
        MarsAsset::Cw20 { contract_addr } => { 
            let trade_msg =  trade_native_for_cw20_on_astroport(deps.as_ref(), config.astroport_router.to_string(), config.stable_denom, ust_amount.into(), deps.api.addr_validate(&contract_addr)? )?;  
            response = response
            .add_message(trade_msg)
            .add_attribute("astroport_buy_cw20_for_ust", contract_addr.to_string());
        },
        MarsAsset::Native { denom } => { 
            if denom != config.stable_denom {
                let trade_msg = trade_native_for_native_on_astroport(deps.as_ref(), config.astroport_router.to_string(), config.stable_denom, ust_amount.into(), denom.clone())?;  
                response = response
                .add_message(trade_msg)
                .add_attribute("astroport_buy_native_for_ust", denom);
    
            }
        },
        _ => { return Err(StdError::generic_err("Invalid debt asset provided") ); }
    }

    // Callback Cosmos Msg 
    let after_masset_buy_callback_msg = CallbackMsg::AfterDebtAssetBuyCallback {
        user_address: user_address,
        debt_asset: debt_asset,
        collateral_asset: collateral_asset,
        ust_amount: ust_amount,
        max_loss_amount: max_loss_amount.into(),
    }.to_cosmos_msg(&env.contract.address)?;
    response = response.add_message(after_masset_buy_callback_msg);

    Ok(response)
}



pub fn after_debt_asset_callback(
    deps: DepsMut,
    env: Env,
    user_address: String,
    debt_asset: MarsAsset,
    collateral_asset: MarsAsset,
    ust_amount: Uint256,
    max_loss_amount: Uint256
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

   // Init response
   let mut response = Response::new().add_attribute("Action", "Mars_Liquidation");
   response = response.add_attribute("user_being_liquidated", user_address.to_string());

    // COSMOS MSGS ::
    // 1. Add LiquidatePosition Msg
    // 2. Add 'AfterLiquidationCallback' Callback Msg  

    let debt_asset_balance : Uint256;
    match debt_asset.clone() {
        MarsAsset::Cw20 { contract_addr }  => {
            debt_asset_balance = query_token_balance(&deps.querier, deps.api.addr_validate( &contract_addr.clone() )? , env.contract.address.clone() )?.into();
            let liquidate_msg =  build_liquidate_cw20_asset_loan_on_red_bank(config.red_bank_addr.to_string(), collateral_asset.clone() ,contract_addr.clone(), debt_asset_balance, user_address )?;
            response = response
            .add_message(liquidate_msg)
            .add_attribute("liquidate_cw20_loan", contract_addr.to_string());

        },
        MarsAsset::Native { denom }  => {
            debt_asset_balance = query_balance(&deps.querier, env.contract.address.clone(),  denom.clone() )?.into();
            let liquidate_msg =  build_liquidate_native_asset_loan_on_red_bank(deps.as_ref(), config.red_bank_addr.to_string(), collateral_asset.clone() , denom.clone(),  debt_asset_balance, user_address )?;
            response = response
            .add_message(liquidate_msg)
            .add_attribute("liquidate_native_loan", denom.to_string());
        }
    }

    let after_liquidation_callback_msg = CallbackMsg::AfterLiquidationCallback {
        debt_asset: debt_asset,
        collateral_asset: collateral_asset,
        ust_amount: ust_amount,
        max_loss_amount: max_loss_amount,    
    }.to_cosmos_msg(&env.contract.address)?;
    response = response.add_message(after_liquidation_callback_msg);

    Ok(response)
}






pub fn after_liquidation_callback(
    deps: DepsMut,
    env: Env,
    debt_asset: MarsAsset,
    collateral_asset: MarsAsset,
    ust_amount: Uint256,
    max_loss_amount: Uint256
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // Init response
    let mut response = Response::new().add_attribute("Action", "Mars_Liquidation");

    // Query minted mAsset & returned collateral asset balance
    // COSMOS MSGS ::
    // 1. Sell debt asset (if any) for UST 
    // 2. Sell received collateral asset UST
    // 3. Add 'AfterAssetsSellCallback' Callback Msg  

    let debt_asset_balance : Uint256;
    match debt_asset {
        MarsAsset::Cw20 { contract_addr }  => {
            debt_asset_balance = query_token_balance(&deps.querier, deps.api.addr_validate( &contract_addr.clone() )? , env.contract.address.clone() )?.into();
            let trade_msg = trade_cw20_for_native_on_astroport(config.astroport_router.to_string(), deps.api.addr_validate( &contract_addr.clone() )? ,debt_asset_balance.into(), config.stable_denom.clone() )?;
            response = response
            .add_message(trade_msg)
            .add_attribute("after_liquidation_sell_cw20_debt", contract_addr.to_string());

        },
        MarsAsset::Native { denom }  => {
            if denom.clone() != config.stable_denom.clone() {
                debt_asset_balance = query_balance(&deps.querier, env.contract.address.clone(),  denom.clone() )?.into();
                let trade_msg = trade_native_for_native_on_astroport(deps.as_ref(),config.astroport_router.to_string(), denom.clone(),  debt_asset_balance.into(), config.stable_denom.clone() )?;
                response = response
                .add_message(trade_msg)
                .add_attribute("after_liquidation_sell_native_debt", denom.to_string());
    
            }
        }
    }

    let collateral_asset_balance : Uint256;
    match collateral_asset {
        MarsAsset::Cw20 { contract_addr }  => {
            collateral_asset_balance = query_token_balance(&deps.querier, deps.api.addr_validate( &contract_addr.clone() )? , env.contract.address.clone() )?.into();
            let trade_msg = trade_cw20_for_native_on_astroport(config.astroport_router.to_string(), deps.api.addr_validate( &contract_addr.clone() )? ,collateral_asset_balance.into(), config.stable_denom.clone() )?;
            response = response
            .add_message(trade_msg)
            .add_attribute("after_liquidation_sell_cw20_collateral", contract_addr.to_string());
    },
        MarsAsset::Native { denom }  => {
            if denom.clone() != config.stable_denom {
                collateral_asset_balance = query_balance(&deps.querier, env.contract.address.clone(),  denom.clone() )?.into();
                let trade_msg = trade_native_for_native_on_astroport(deps.as_ref(), config.astroport_router.to_string(), denom.clone(),  collateral_asset_balance.into(), config.stable_denom )?;
                response = response
                .add_message(trade_msg)
                .add_attribute("after_liquidation_sell_native_collateral", denom.to_string());
            }
        }
    }

    let after_asset_sell_callback_msg = CallbackMsg::AfterAssetsSellCallback {
        ust_amount: ust_amount,
        max_loss_amount: max_loss_amount,    
    }.to_cosmos_msg(&env.contract.address)?;
    response = response.add_message(after_asset_sell_callback_msg);

    Ok(response)
}



pub fn after_assets_sell_callback(
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
    
    let profit = computed_ust_balance - min_ust_balance;
    state.total_liquidations += 1u64;
    state.total_ust_profit += profit;

    // COSMOS MSGS :: 
    // 1. Send UST Back to the UST arb strategy
    // 2. Update Indexes and deposit UST Back into Anchor
    let send_native_asset_msg = build_send_native_asset_msg( deps.as_ref(), config.ust_vault_address.clone(), &config.stable_denom, cur_ust_balance.into() )?;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
    .add_message(send_native_asset_msg)
    .add_attribute("return_ust_to_vault", cur_ust_balance.to_string() )
    .add_attribute("profit", profit.to_string() ))
}



//-----------------------------------------------------------
// QUERY HANDLERS
//-----------------------------------------------------------


/// @dev Returns the contract's configuration
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        ust_vault_address: config.ust_vault_address.to_string(),
        red_bank_addr: config.red_bank_addr.to_string(),
        astroport_router:  config.astroport_router.to_string(),
        stable_denom: config.stable_denom,
        assets_supported: config.assets_supported
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



/// @dev query maAsset info from the Red Bank contract
pub fn query_market(
    querier: &QuerierWrapper,
    redbank_addr: String,
    masset: MarsAsset
) -> StdResult< RedBankMarketQueryResponse > {
    let res: RedBankMarketQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: redbank_addr,
            msg: to_binary(&MarsQueryMsg::Market {
                asset: masset
            })?,
        }))?;
    Ok(res)
}


//-----------------------------------------------------------
// HELPER FUNCTIONS :: COSMOS MSGs
//-----------------------------------------------------------

fn build_liquidate_native_asset_loan_on_red_bank(
    deps: Deps,
    red_bank_addr: String,
    collateral_asset: MarsAsset,
    debt_asset_denom: String,
    amount_to_repay: Uint256,
    user_address: String,
) -> StdResult<CosmosMsg> {

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:red_bank_addr,
        funds: vec![deduct_tax(  deps,  Coin {   denom: debt_asset_denom.to_string(),  amount: amount_to_repay.into(),}, )? ],
        msg: to_binary(&MarsLiquidationMsg::LiquidateNative {
            collateral_asset: collateral_asset,
            debt_asset_denom: debt_asset_denom,
            user_address: user_address,
            receive_ma_token: false
        })?,
    }))
}


fn build_liquidate_cw20_asset_loan_on_red_bank(
    red_bank_addr: String,
    collateral_asset: MarsAsset,
    debt_asset_addr: String,
    amount_to_repay: Uint256,
    user_address: String,
) -> StdResult<CosmosMsg> {

    let msg_ = to_binary(&MarsLiquidationMsg::LiquidateCw20 {
        collateral_asset: collateral_asset,
        user_address: user_address,
        receive_ma_token: false,
    })?;

    Ok(build_send_cw20_token_msg(red_bank_addr,debt_asset_addr,amount_to_repay.into(),msg_ )?)
}



fn build_mint_nft_msg(
    nft_minter: String,
    user_address: String,
    liquidated_amount: Uint256
) -> StdResult<CosmosMsg> {

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr:nft_minter,
        funds: vec![],
        msg: to_binary(&MinterExecuteMsg::MintNft {
            user_address: user_address,
            liquidated_amount: liquidated_amount
        })?,
    }))
}





