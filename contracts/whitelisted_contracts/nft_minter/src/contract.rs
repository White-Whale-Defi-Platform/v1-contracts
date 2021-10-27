#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, StdError, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg
};

use whitewhale_liquidation_helpers::nft_minter::{
    InstantiateMsg, ExecuteMsg, QueryMsg, ConfigResponse, LiquidationHelpersInfo
};

use crate::state::{ Config, CONFIG};
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
            whitewhale_liquidators: vec![],
    };

    CONFIG.save(deps.storage, &config)?;
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
        ExecuteMsg::UpdateOwner { owner } => handle_update_owner(deps, info, owner),
        ExecuteMsg::AddLiquidator { new_liquidator } => handle_add_liquidator(deps, info, new_liquidator),
        ExecuteMsg::MintNft { user_address, liquidated_amount } => handle_mint_nft(deps, _env, info, user_address, liquidated_amount ),
    }
}





#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}




//----------------------------------------------------------------------------------------
// EXECUTE FUNCTION HANDLERS
//----------------------------------------------------------------------------------------


/// @dev Admin function to update Configuration parameters
/// @param new_config : Same as UpdateConfigMsg struct
pub fn handle_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // UPDATE :: ADDRESSES IF PROVIDED
    config.owner =  deps.api.addr_validate( &new_owner )?; 

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "nft_minter::ExecuteMsg::UpdateOwner"))
}


/// @dev Admin function to add a new Fields strategy address
/// @param new_asset : 
pub fn handle_add_liquidator(
    deps: DepsMut,
    info: MessageInfo,
    new_liquidator: LiquidationHelpersInfo
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner.clone() {
        return Err(StdError::generic_err("Unauthorized"));
    }

    for whitewhale_liquidator in config.whitewhale_liquidators.iter() {
        if new_liquidator.liquidator_contract == whitewhale_liquidator.liquidator_contract {
            return Err(StdError::generic_err("Already Supported"));
        }
    }

    config.whitewhale_liquidators.push(  new_liquidator );

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "nft_minter::ExecuteMsg::AddLiquidator"))
}




/// @dev 
/// @param  : 
pub fn handle_mint_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
    liquidated_amount: Uint256
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let mut is_valid_call = false;
    for whitewhale_liquidator in config.whitewhale_liquidators.iter() {
        if info.sender.to_string() == whitewhale_liquidator.liquidator_contract {
            is_valid_call = true;
            break;
        }
    }

    // let flash_loan_msg = build_flash_loan_msg( config.ust_vault_address.to_string(),
    //                                             config.stable_denom,
    //                                             ust_to_borrow,
    //                                             callback_binary )?;

    Ok(Response::new()
    // .add_message(flash_loan_msg)
    .add_attribute("action", "nft_minter::ExecuteMsg::MintNFT"))
                                            
}


//-----------------------------------------------------------
// QUERY HANDLERS
//-----------------------------------------------------------


/// @dev Returns the contract's configuration
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        whitewhale_liquidators: config.whitewhale_liquidators,
    })
}





//-----------------------------------------------------------
// HELPER FUNCTIONS :: COSMOS MSGs
//-----------------------------------------------------------


// fn build_mint_msg(
//     nft_addr: String,
//     user_addr: String,
// ) -> StdResult<CosmosMsg> {

//     Ok(CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: fields_addr,
//         funds: vec![],
//         msg: to_binary(&MartianFieldsLiquidationMsg::Liquidate {
//             user: user_addr
//         })?,
//     }))

// }
