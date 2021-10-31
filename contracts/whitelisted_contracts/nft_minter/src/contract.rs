#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, StdError, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg, ReplyOn, SubMsg, Reply, Addr, CosmosMsg
};

// use schemars::schema::Metadata;
use whitewhale_liquidation_helpers::nft_minter::{
    InstantiateMsg, ExecuteMsg, QueryMsg, ConfigResponse, LiquidationHelpersInfo
};
use whitewhale_liquidation_helpers::metadata::{Metadata,Trait};
use whitewhale_liquidation_nft::msg::{InstantiateMsg as Cw721InstantiateMsg, ExecuteMsg as Cw721ExecuteMsg, MintMsg as Cw721MintMsg};
use crate::response::MsgInstantiateContractResponse;
use protobuf::Message;
use crate::state::{ Config, TmpNftInfo, CONFIG, TMP_NFT_INFO};
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
            cw721_code_id: msg.cw721_code_id,
            whitewhale_liquidators: vec![],
    };

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateOwner { owner } => handle_update_owner(deps, info, owner),
        ExecuteMsg::AddLiquidator { new_liquidator, metadata, symbol, token_uri } => handle_initialize_liquidator(deps, env, info, new_liquidator, metadata, symbol, token_uri),
        ExecuteMsg::UpdateLiquidator { cur_liquidator, new_liquidator , metadata, token_uri } => handle_update_metadata(deps, env, info, cur_liquidator, new_liquidator , metadata, token_uri),
        ExecuteMsg::MintNft { user_address, liquidated_amount } => handle_mint_nft(deps, env, info, user_address, liquidated_amount ),
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



pub fn handle_initialize_liquidator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_liquidator: String,
    metadata: Metadata,
    symbol: String,
    token_uri: String
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner.clone() {
        return Err(StdError::generic_err("Unauthorized"));
    }

    for whitewhale_liquidator in config.whitewhale_liquidators.iter() {
        if new_liquidator == whitewhale_liquidator.liquidator_contract {
            return Err(StdError::generic_err("Liquidator already Supported"));
        }
    }

    TMP_NFT_INFO.save(
        deps.storage,
        &TmpNftInfo {
            liquidator_addr: new_liquidator.clone(),
            metadata: metadata.clone(),
            token_uri: token_uri.clone()
        },
    )?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "add_liquidator"),
            ("liquidator_address", &new_liquidator),
            ("nft_symbol", &symbol),
        ])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.cw721_code_id,
                funds: vec![],
                admin: None,
                label: "".to_string(),
                msg: to_binary(&Cw721InstantiateMsg {
                           name: metadata.name.unwrap(),
                           symbol: symbol,
                           minter: env.contract.address.to_string(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}


/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let tmp_nft_info = TMP_NFT_INFO.load(deps.storage)?;

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(msg.result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let nft_contract = res.get_contract_address();

    let mut config = CONFIG.load(deps.storage)?;
    config.whitewhale_liquidators.push(  LiquidationHelpersInfo {
        liquidator_contract: tmp_nft_info.liquidator_addr,
        nft_contract_addr: nft_contract.to_string(),
        total_minted: 0u64,
        metadata: tmp_nft_info.metadata,
        token_uri: tmp_nft_info.token_uri
    } );


    CONFIG.save(deps.storage,&config)?;

    Ok(Response::new().add_attributes(vec![
        ("nft_address", nft_contract.to_string()),
    ]))
}



pub fn handle_update_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cur_liquidator: String,
    new_liquidator: Option<String>,
    metadata: Option<Metadata>,
    token_uri: Option<String>
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut check = false;

    // CHECK :: Only owner can call this function
    if info.sender != config.owner.clone() {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let mut response = Response::new().add_attributes(vec![
        ("action", "updated_Liquidator"), ("liquidator", "cur_liquidator"),
    ]);

    for mut whitewhale_liquidator in config.whitewhale_liquidators.iter_mut() {
        if cur_liquidator == whitewhale_liquidator.liquidator_contract {
            if let Some(new_liquidator) = new_liquidator.clone() {
                whitewhale_liquidator.liquidator_contract = new_liquidator.clone();
                response = response.add_attribute("new_liquidator", new_liquidator.clone());
            }
            if let Some(metadata) = metadata.clone() {
                whitewhale_liquidator.metadata = metadata;
                response = response.add_attribute("metadata_updated", "true");
            }
            if let Some(token_uri) = token_uri.clone() {
                whitewhale_liquidator.token_uri = token_uri.clone();
                response = response.add_attribute("updated_token_uri", token_uri);
            }
            check = true;
        }
    }

    if !check {
        return Err(StdError::generic_err("Liquidator not Supported"));
    }
    CONFIG.save(deps.storage,&config)?;
    Ok(response)
}



pub fn handle_mint_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
    liquidated_amount: Uint256
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    let mut response = Response::new().add_attributes(vec![
        ("action", "mint_nft"), ("liquidator", &info.sender.to_string()),
    ]);

    let mut is_valid_call = false;
    for mut whitewhale_liquidator in config.whitewhale_liquidators.iter_mut() {
        if info.sender.to_string() == whitewhale_liquidator.liquidator_contract {
            let nft_mint_msg = build_mint_msg(
                env.block.time.seconds(),
                whitewhale_liquidator.nft_contract_addr.clone().to_string(),
                whitewhale_liquidator.total_minted.clone() + 1u64,
                whitewhale_liquidator.token_uri.clone(),
                whitewhale_liquidator.metadata.clone(),
                user_address,
                liquidated_amount.clone(),
            )?;
            response = response.add_message(nft_mint_msg);
            whitewhale_liquidator.total_minted +=1u64;
            is_valid_call = true;
            break;
        }
    }

    if !is_valid_call  {
        return Err(StdError::generic_err("Liquidator not Supported"));
    }


    CONFIG.save(deps.storage,&config)?;

    Ok(response)                        
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


/// Helper Function. Returns CosmosMsg to mint the NFT
pub fn build_mint_msg(
    timestamp: u64,
    nft_contract_addr: String,
    token_id: u64,
    token_uri: String,
    metadata: Metadata,
    user_addr: String,
    liquidated_amount: Uint256
) -> StdResult<CosmosMsg> {

    let mut attributes_vec = vec![];
    let user_attribute = Trait {
        display_type: None,
        trait_type: "liquidated_user".to_string(),
        value: user_addr.clone(),
    };
    attributes_vec.push(user_attribute);

    let amount_attribute = Trait {
        display_type: None,
        trait_type: "liquidated_amount".to_string(),
        value: liquidated_amount.to_string(),
    };
    attributes_vec.push(amount_attribute);

    let time_attribute = Trait {
        display_type: None,
        trait_type: "timestamp".to_string(),
        value: timestamp.to_string(),
    };
    attributes_vec.push(time_attribute);



    let extension_ = Metadata {
        image: metadata.image.clone(),
        image_data: None,
        external_url: None,
        description: metadata.description.clone(),
        name: Some(metadata.name.clone().unwrap() + &" #".to_string() + &token_id.to_string()),
        attributes: Some(attributes_vec),
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };

    let mint_msg = Cw721MintMsg {
        token_id: token_id.to_string(),
        owner: user_addr,
        name: metadata.name.unwrap() + &" #".to_string() + &token_id.to_string(),
        description: metadata.description,
        image: Some(token_uri),
        extension: extension_,
    };

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nft_contract_addr.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::Mint(mint_msg))?,
        funds: vec![],
    }))
}