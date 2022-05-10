use cosmwasm_std::{
    entry_point, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError,
    SubMsg, Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::MinterResponse;
use protobuf::Message;
use semver::Version;
use serde::Serialize;
use terraswap::asset::AssetInfo;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use white_whale::denom::LUNA_DENOM;
use white_whale::deposit_info::DepositInfo;
use white_whale::fee::{Fee, VaultFee};
use white_whale::luna_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::luna_vault::msg::*;

use crate::commands::set_fee;
use crate::error::LunaVaultError;
use crate::helpers::{get_lp_token_address, unwrap_data, unwrap_reply};
use crate::pool_info::PoolInfoRaw;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{ProfitCheck, State, ADMIN, DEPOSIT_INFO, FEE, POOL_INFO, PROFIT, STATE};
use crate::{commands, flashloan, helpers, queries, replies};

const INSTANTIATE_REPLY_ID: u64 = 1u64;
pub(crate) const INSTANTIATE_UNBOND_HANDLER_REPLY_ID: u64 = 2u64;
pub const DEFAULT_LP_TOKEN_NAME: &str = "White Whale Luna Vault LP Token";
pub const DEFAULT_LP_TOKEN_SYMBOL: &str = "wwVLuna";

pub(crate) type VaultResult<T> = Result<T, LunaVaultError>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ww-luna-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> VaultResult<Response> {
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let astro_lp_address = deps.api.addr_validate(&msg.astro_lp_address)?;
    let astro_factory_address = deps.api.addr_validate(&msg.astro_factory_address)?;

    let state = State {
        bluna_address: deps.api.addr_validate(&msg.bluna_address)?,
        cluna_address: deps.api.addr_validate(&msg.cluna_address)?,
        astro_lp_address: astro_lp_address.clone(),
        astro_factory_address,
        memory_address: deps.api.addr_validate(&msg.memory_addr)?,
        whitelisted_contracts: vec![],
        allow_non_whitelisted: false,
        unbond_handler_code_id: msg.unbond_handler_code_id,
    };

    // Store the initial config
    STATE.save(deps.storage, &state)?;

    // Check if the provided asset is the luna token
    let _underlying_coin_denom = match msg.asset_info.clone() {
        AssetInfo::Token { .. } => return Err(LunaVaultError::NotNativeToken {}),
        AssetInfo::NativeToken { denom } => {
            if denom != LUNA_DENOM {
                return Err(LunaVaultError::NotLunaToken {});
            }
            denom
        }
    };

    DEPOSIT_INFO.save(
        deps.storage,
        &DepositInfo {
            asset_info: msg.asset_info.clone(),
        },
    )?;
    // Setup the fees system with a fee and other contract addresses
    let fee_config = VaultFee {
        flash_loan_fee: helpers::check_fee(Fee {
            share: msg.flash_loan_fee,
        })?,
        treasury_fee: helpers::check_fee(Fee {
            share: msg.treasury_fee,
        })?,
        commission_fee: helpers::check_fee(Fee {
            share: msg.commission_fee,
        })?,
        treasury_addr: deps.api.addr_validate(&msg.treasury_addr)?,
    };

    FEE.save(deps.storage, &fee_config)?;
    println!("Start");

    let pool_info: &PoolInfoRaw = &PoolInfoRaw {
        contract_addr: env.contract.address.clone(),
        liquidity_token: Addr::unchecked(""),
        asset_infos: [
            msg.asset_info.to_raw(deps.api)?, // 0 - luna
            AssetInfo::Token {
                contract_addr: get_lp_token_address(&deps.as_ref(), astro_lp_address)?
                    .into_string(),
            }
            .to_raw(deps.api)?, // 1 - astro lp
            AssetInfo::Token {
                contract_addr: msg.bluna_address,
            }
            .to_raw(deps.api)?, // 2 - bluna
            AssetInfo::Token {
                contract_addr: msg.cluna_address,
            }
            .to_raw(deps.api)?, // 3 - cluna
        ],
    };
    println!("Start");

    POOL_INFO.save(deps.storage, pool_info)?;

    let profit = ProfitCheck {
        last_balance: Uint128::zero(),
        last_profit: Uint128::zero(),
    };
    PROFIT.save(deps.storage, &profit)?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    // Both the lp_token_name and symbol are Options, attempt to unwrap their value falling back to the default if not provided
    let lp_token_name: String = msg
        .vault_lp_token_name
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));
    let lp_token_symbol: String = msg
        .vault_lp_token_symbol
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

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
            label: "White Whale Luna Vault LP".to_string(),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VaultResult<Response> {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> VaultResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity { asset } => {
            commands::provide_liquidity(deps, env, info, asset)
        }
        ExecuteMsg::WithdrawUnbonded {} => commands::withdraw_unbonded(deps, info, false, None),
        ExecuteMsg::WithdrawUnbondedFlashloan {} => {
            commands::withdraw_unbonded_from_flashloan(deps, info, env)
        }
        ExecuteMsg::SetAdmin { admin } => commands::set_admin(deps, info, admin),
        ExecuteMsg::SetFee {
            flash_loan_fee,
            treasury_fee,
            commission_fee,
        } => set_fee(deps, info, flash_loan_fee, treasury_fee, commission_fee),
        ExecuteMsg::AddToWhitelist { contract_addr } => {
            commands::add_to_whitelist(deps, info, contract_addr)
        }
        ExecuteMsg::RemoveFromWhitelist { contract_addr } => {
            commands::remove_from_whitelist(deps, info, contract_addr)
        }
        ExecuteMsg::FlashLoan { payload } => flashloan::handle_flashloan(deps, env, info, payload),
        ExecuteMsg::SwapRewards {} => commands::swap_rewards(deps, env, info),
        ExecuteMsg::UpdateState {
            bluna_address,
            cluna_address,
            astro_lp_address,
            memory_address,
            whitelisted_contracts,
            allow_non_whitelisted,
        } => commands::update_state(
            deps,
            info,
            bluna_address,
            cluna_address,
            astro_lp_address,
            memory_address,
            whitelisted_contracts,
            allow_non_whitelisted,
        ),
        ExecuteMsg::Callback(msg) => flashloan::_handle_callback(deps, env, info, msg),
        ExecuteMsg::UnbondHandler(msg) => commands::handle_unbond_handler_msg(deps, info, msg),
        ExecuteMsg::LiquidateExpiredUnbondHandler {
            liquidate_unbond_handler_addr,
        } => commands::withdraw_unbonded(deps, info, true, Some(liquidate_unbond_handler_addr)),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> VaultResult<Response> {
    let res = unwrap_reply(msg.clone())?;
    let data = unwrap_data(res.clone())?;

    let response: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
        .map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    match msg.id {
        INSTANTIATE_REPLY_ID => replies::after_token_instantiation(deps, response),
        INSTANTIATE_UNBOND_HANDLER_REPLY_ID => {
            let events = res.events;
            replies::after_unbond_handler_instantiation(deps, response, events)
        }
        _ => Ok(Response::default()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> VaultResult<Binary> {
    match msg {
        QueryMsg::PoolConfig {} => to_binary(&queries::query_pool_info(deps)?),
        QueryMsg::PoolState {} => to_binary(&queries::try_query_pool_state(env, deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::Fees {} => to_binary(&queries::query_fees(deps)?),
        QueryMsg::VaultValue {} => to_binary(&queries::query_total_value(env, deps)?),
        QueryMsg::EstimateWithdrawFee { amount } => {
            to_binary(&queries::estimate_withdraw_fee(deps, amount)?)
        }
        QueryMsg::LastBalance {} => to_binary(&queries::query_last_balance(deps)?),
        QueryMsg::LastProfit {} => to_binary(&queries::query_last_profit(deps)?),
        QueryMsg::WithdrawableUnbonded { address } => {
            to_binary(&queries::query_withdrawable_unbonded(deps, address)?)
        }
        QueryMsg::UnbondRequests { address } => {
            to_binary(&queries::query_unbond_requests(deps, address)?)
        }
        QueryMsg::UnbondHandlerExpirationTime {} => to_binary(
            &queries::query_unbond_handler_expiration_time(deps.storage)?,
        ),
    }
}

fn to_binary<T>(data: &T) -> VaultResult<Binary>
where
    T: Serialize + ?Sized,
{
    Ok(cosmwasm_std::to_binary(data)?)
}
