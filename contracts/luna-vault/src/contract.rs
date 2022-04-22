use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::MinterResponse;
use protobuf::Message;
use semver::Version;
use terraswap::asset::AssetInfo;

use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use white_whale::denom::LUNA_DENOM;
use white_whale::deposit_info::DepositInfo;
use white_whale::fee::{Fee, VaultFee};
use white_whale::luna_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::luna_vault::msg::*;

use crate::commands::set_fee;
use crate::error::LunaVaultError;
use crate::helpers::get_lp_token_address;
use crate::{commands, flashloan, helpers, queries};

use crate::pool_info::PoolInfoRaw;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{
    CurrentBatch, ProfitCheck, State, ADMIN, CURRENT_BATCH, DEPOSIT_INFO, FEE, POOL_INFO, PROFIT,
    STATE,
};

const INSTANTIATE_REPLY_ID: u8 = 1u8;
pub const DEFAULT_LP_TOKEN_NAME: &str = "White Whale Luna Vault LP Token";
pub const DEFAULT_LP_TOKEN_SYMBOL: &str = "wwVLuna";

pub(crate) type VaultResult = Result<Response, LunaVaultError>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ww-luna-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> VaultResult {
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let astro_lp_address = deps.api.addr_validate(&msg.astro_lp_address)?;

    let state = State {
        bluna_address: deps.api.addr_validate(&msg.bluna_address)?,
        astro_lp_address: astro_lp_address.clone(),
        memory_address: deps.api.addr_validate(&msg.memory_addr)?,
        whitelisted_contracts: vec![],
        allow_non_whitelisted: false,
        unbonding_period: msg.unbonding_period,
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

    let pool_info: &PoolInfoRaw = &PoolInfoRaw {
        contract_addr: env.contract.address.clone(),
        liquidity_token: Addr::unchecked(""),
        luna_cap: msg.luna_cap,
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
    POOL_INFO.save(deps.storage, pool_info)?;

    let profit = ProfitCheck {
        last_balance: Uint128::zero(),
        last_profit: Uint128::zero(),
    };
    PROFIT.save(deps.storage, &profit)?;

    // Setup current batch
    let batch = CurrentBatch { id: 1 };
    CURRENT_BATCH.save(deps.storage, &batch)?;

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
        id: u64::from(INSTANTIATE_REPLY_ID),
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VaultResult {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> VaultResult {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity { asset } => {
            commands::provide_liquidity(deps, env, info, asset)
        }
        ExecuteMsg::WithdrawUnbonded {} => commands::withdraw_unbonded(deps, env, info),
        ExecuteMsg::SetLunaCap { luna_cap } => commands::set_luna_cap(deps, info, luna_cap),
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
            astro_lp_address,
            memory_address,
            whitelisted_contracts,
            allow_non_whitelisted,
            unbonding_period,
        } => commands::update_state(
            deps,
            info,
            bluna_address,
            astro_lp_address,
            memory_address,
            whitelisted_contracts,
            allow_non_whitelisted,
            unbonding_period,
        ),
        ExecuteMsg::Callback(msg) => flashloan::_handle_callback(deps, env, info, msg),
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
        POOL_INFO.update(deps.storage, |mut meta| -> StdResult<_> {
            meta.liquidity_token = api.addr_validate(liquidity_token)?;
            Ok(meta)
        })?;

        return Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token));
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
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
        QueryMsg::CurrentBatch {} => to_binary(&queries::query_current_batch(deps)?),
        QueryMsg::WithdrawableUnbonded { address } => {
            to_binary(&queries::query_withdrawable_unbonded(deps, address, env)?)
        }
        QueryMsg::UnbondRequests {
            address,
            start_from,
            limit,
        } => to_binary(&queries::query_unbond_requests(
            deps, address, start_from, limit,
        )?),
        QueryMsg::AllHistory { start_from, limit } => to_binary(
            &queries::query_unbond_requests_limitation(deps, start_from, limit)?,
        ),
    }
}
