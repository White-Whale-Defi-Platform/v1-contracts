#![allow(unused_imports)]
#![allow(unused_variables)]
use std::vec;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Reply, Deps, ReplyOn, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
    WasmMsg, Addr, StdError,
};
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use white_whale::fee::Fee;
use white_whale::treasury::dapp_base::commands as dapp_base_commands;

use white_whale::treasury::dapp_base::common::BaseDAppResult;
use white_whale::treasury::dapp_base::msg::BaseInstantiateMsg;
use white_whale::treasury::dapp_base::queries as dapp_base_queries;
use white_whale::treasury::dapp_base::state::{BaseState, ADMIN};

use crate::response::MsgInstantiateContractResponse;

use crate::commands;
use crate::error::PoolingError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, POOL, Pool, FEE};
pub type PoolingResult = Result<Response, PoolingError>;

const INSTANTIATE_REPLY_ID: u8 = 1u8;

const DEFAULT_LP_TOKEN_NAME: &str = "Vault LP token";
const DEFAULT_LP_TOKEN_SYMBOL: &str = "uvLP";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> PoolingResult {
    let base_state: BaseState = dapp_base_commands::handle_base_init(deps.as_ref(), msg.base)?;
    
    let state: State = State{
        base: base_state,
        lp_token_addr: Addr::unchecked("")
    };

    let lp_token_name: String = msg
        .vault_lp_token_name
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));

    let lp_token_symbol: String = msg
        .vault_lp_token_symbol
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

    STATE.save(deps.storage, &state)?;
    POOL.save(deps.storage, &Pool{
        deposit_asset: msg.deposit_asset,
        assets: vec![msg.deposit_asset],
    });
    FEE.save(deps.storage, &Fee{
        share: msg.fee 
    });
    ADMIN.set(deps, Some(info.sender))?;
    
    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: lp_token_name,
                symbol: lp_token_symbol,
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })?,
            funds: vec![],
            label: "White Whale Vault LP".to_string(),
        }
        .into(),
        gas_limit: None,
        id: u64::from(INSTANTIATE_REPLY_ID),
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> PoolingResult {
    match msg {
        ExecuteMsg::Base(message) => from_base_dapp_result(dapp_base_commands::handle_base_message(deps, info, message)),
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity { asset } => commands::try_provide_liquidity(deps, info, asset),
        ExecuteMsg::UpdateState {
        } => commands::update_state(
            deps,
            info,
            anchor_money_market_address,
            aust_address,
            profit_check_address,
            allow_non_whitelisted,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => dapp_base_queries::handle_base_query(deps, message),
        // handle dapp-specific queries here
        QueryMsg::State{} => to_binary(&STATE.load(deps.storage)?),
    }
}


/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    if msg.id == u64::from(INSTANTIATE_REPLY_ID) {
        let data = msg.result.unwrap().data.unwrap();
        let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
            .map_err(|_| {
                StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
            })?;
        let liquidity_token = res.get_contract_address();

        let api = deps.api;
        STATE.update(deps.storage, |mut meta| -> StdResult<_> {
            meta.lp_token_addr = api.addr_validate(liquidity_token)?;
            Ok(meta)
        })?;

        return Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token));
    }
    Ok(Response::default())
}

/// Required to convert BaseDAppResult into TerraswapResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: BaseDAppResult) -> PoolingResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}