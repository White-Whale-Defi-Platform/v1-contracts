use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdResult, Uint128, WasmMsg, WasmQuery, Uint128
};
use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ReceiveMsg};
use cw20_base::allowances::{
    execute_decrease_allowance, execute_increase_allowance, query_allowance,
};
use cw20_base::contract::{
    create_accounts, execute_update_marketing, execute_upload_logo, query_balance,
    query_download_logo, query_marketing_info, query_minter, query_token_info,
};
use cw20_base::enumerable::{query_all_accounts, query_all_allowances};
use cw20_base::state::{BALANCES, TOKEN_INFO};
use cw20_base::ContractError;

use mars_core::cw20_core::instantiate_token_info_and_marketing;
use mars_core::red_bank;

use crate::allowances::{execute_send_from, execute_transfer_from};
use crate::core;
use crate::msg::{BalanceAndTotalSupplyResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::CONFIG;
use crate::Config;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:staked-whale-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ************ CHECK MATH ************
/// ************ CHECK MATH ************

const MAX_UINT128 = Uint128::MAX;
const INITIAL_FRAGMENTS_SUPPLY = 5000000 * 10**6;

// TOTAL_GONS is a multiple of INITIAL_FRAGMENTS_SUPPLY so that _gonsPerFragment is an integer.
// Use the highest value that fits in a uint256 for max granularity.
const TOTAL_GONS = MAX_UINT128 - (MAX_UINT128 % INITIAL_FRAGMENTS_SUPPLY);

// MAX_SUPPLY = maximum integer < (sqrt(4*TOTAL_GONS + 1) - 1) / 2
const MAX_SUPPLY = = Uint128::MAX; // (2^128) - 1

/// ************ CHECK MATH ************
/// ************ CHECK MATH ************


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let base_msg = cw20_base::msg::InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        initial_balances: msg.initial_balances,
        mint: msg.mint,
        marketing: msg.marketing,
    };
    base_msg.validate()?;

    let total_supply = create_accounts(&mut deps, &base_msg.initial_balances)?;
    instantiate_token_info_and_marketing(&mut deps, base_msg, total_supply)?;

    // store token config
    CONFIG.save(
        deps.storage,
        &Config {
            initializer: deps.api.addr_validate(&msg.initializer)?,
            staking_contract_address: deps.api.addr_validate(&msg.staking_contract_address)?,
            rebase_tracker: 0u64,
        },
    )?;

    let mut res = Response::new();

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { user, amount } => execute_burn(deps, env, info, user, amount),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => execute_send_from(deps, env, info, owner, contract, amount, msg),
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => execute_upload_logo(deps, env, info, logo),
    }
}



pub fn execute_initialize(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    staking_contract: String
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if config.initializer != msg.sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    config.staking_contract_address = deps.api.addr_validate(&staking_contract)?;
    config.initializer = None;

    let mut gon_balance = GON_BALANCES.may_load(deps.storage, &config.staking_contract_address).unwrap_or_default();   
    gon_balance = TOTAL_GONS;



}


pub fn execute_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient_unchecked: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let config = CONFIG.load(deps.storage)?;

    let recipient = deps.api.addr_validate(&recipient_unchecked)?;
    core::transfer(
        deps.storage,
        &config,
        info.sender.clone(),
        recipient,
        amount,
        true,
    )?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient_unchecked)
        .add_attribute("amount", amount)
    Ok(res)
}


pub fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user_unchecked: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // only money market can burn
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.red_bank_address {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // lower balance
    let user_address = deps.api.addr_validate(&user_unchecked)?;
    let user_balance_before = core::decrease_balance(deps.storage, &user_address, amount)?;

    // reduce total_supply
    let mut total_supply_before = Uint128::zero();
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        total_supply_before = info.total_supply;
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("user", user_unchecked)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient_unchecked: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    if token_info.mint.is_none() || token_info.mint.as_ref().unwrap().minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let total_supply_before = token_info.total_supply;

    // update supply and enforce cap
    token_info.total_supply += amount;
    if let Some(limit) = token_info.get_cap() {
        if token_info.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // add amount to recipient balance
    let rcpt_address = deps.api.addr_validate(&recipient_unchecked)?;
    let rcpt_balance_before = core::increase_balance(deps.storage, &rcpt_address, amount)?;

    let config = CONFIG.load(deps.storage)?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient_unchecked)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_unchecked: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // move the tokens to the contract
    let config = CONFIG.load(deps.storage)?;
    let contract_address = deps.api.addr_validate(&contract_unchecked)?;

    core::transfer(
        deps.storage,
        &config,
        info.sender.clone(),
        contract_address,
        amount,
        true,
    )?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", info.sender.to_string())
        .add_attribute("to", &contract_unchecked)
        .add_attribute("amount", amount)
        .add_messages(transfer_messages)
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.to_string(),
                amount,
                msg,
            }
            .into_cosmos_msg(contract_unchecked)?,
        );

    Ok(res)
}

// QUERY

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::BalanceAndTotalSupply { address } => {
            to_binary(&query_balance_and_total_supply(deps, address)?)
        }
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&query_all_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
        QueryMsg::MarketingInfo {} => to_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&query_download_logo(deps)?),
    }
}

fn query_balance_and_total_supply(
    deps: Deps,
    address_unchecked: String,
) -> StdResult<BalanceAndTotalSupplyResponse> {
    let address = deps.api.addr_validate(&address_unchecked)?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    let info = TOKEN_INFO.load(deps.storage)?;
    Ok(BalanceAndTotalSupplyResponse {
        balance,
        total_supply: info.total_supply,
    })
}

