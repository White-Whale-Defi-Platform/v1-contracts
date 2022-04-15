use cosmwasm_std::{Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, entry_point, Env, Fraction, from_binary, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, to_binary, Uint128, WasmMsg, WasmQuery};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg, MinterResponse, TokenInfoResponse};
use protobuf::Message;
use semver::Version;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::asset::AssetInfo::Token;
use terraswap::pair::Cw20HookMsg;
use terraswap::querier::{query_balance, query_supply, query_token_balance};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use white_whale::anchor::{anchor_deposit_msg, anchor_withdraw_msg};
use white_whale::denom::LUNA_DENOM;
use white_whale::deposit_info::DepositInfo;
use white_whale::fee::{Fee, VaultFee};
use white_whale::luna_vault::msg::*;
use white_whale::luna_vault::msg::{
    EstimateWithdrawFeeResponse, FeeResponse, ValueResponse, VaultQueryMsg as QueryMsg,
};
use white_whale::memory::LIST_SIZE_LIMIT;
use white_whale::tax::{compute_tax, into_msg_without_tax};

use crate::{commands, flashloan, helpers, queries};
use crate::commands::set_fee;
use crate::error::LunaVaultError;
use crate::helpers::{compute_total_value, validate_rate};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{ADMIN, CURRENT_BATCH, CurrentBatch, DEPOSIT_INFO, FEE, Parameters, PARAMETERS, POOL_INFO, PROFIT, ProfitCheck, State, STATE};

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
    let state = State {
        bluna_address: deps.api.addr_validate(&msg.bluna_address)?,
        memory_address: deps.api.addr_validate(&msg.memory_addr)?,
        whitelisted_contracts: vec![],
        allow_non_whitelisted: false,

        exchange_rate: Decimal::one(),
        total_bond_amount: Uint128::zero(),
        last_index_modification: env.block.time.seconds(),
        last_unbonded_time: env.block.time.seconds(),
        prev_vault_balance: Uint128::zero(),
        actual_unbonded_amount: Uint128::zero(),
        last_processed_batch: 0u64,
    };

    // Store the initial config
    STATE.save(deps.storage, &state)?;

    // Check if the provided asset is the luna token
    let underlying_coin_denom = match msg.asset_info.clone() {
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
            AssetInfo::Token { contract_addr: msg.astro_lp_address }.to_raw(deps.api)?,  // 1 - astro lp
            AssetInfo::Token { contract_addr: msg.bluna_address }.to_raw(deps.api)?, // 2 - bluna
            AssetInfo::Token { contract_addr: msg.cluna_address }.to_raw(deps.api)? // 3 - cluna
        ],
    };
    POOL_INFO.save(deps.storage, pool_info)?;

    let profit = ProfitCheck {
        last_balance: Uint128::zero(),
        last_profit: Uint128::zero(),
    };
    PROFIT.save(deps.storage, &profit)?;

    // Setup parameters
    let params = Parameters {
        epoch_period: msg.epoch_period,
        underlying_coin_denom,
        unbonding_period: msg.unbonding_period,
        peg_recovery_fee: validate_rate(msg.peg_recovery_fee)?,
        er_threshold: validate_rate(msg.er_threshold)?,
    };
    PARAMETERS.save(deps.storage, &params)?;

    // Setup current batch
    let batch = CurrentBatch {
        id: 1,
        requested_with_fee: Default::default(),
    };
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
        ExecuteMsg::ProvideLiquidity { asset } => commands::provide_liquidity(deps, env, info, asset),
        ExecuteMsg::WithdrawUnbonded {} => commands::execute_withdraw_unbonded(deps, env, info),
        ExecuteMsg::SetLunaCap { luna_cap } => commands::set_luna_cap(deps, info, luna_cap),
        ExecuteMsg::SetAdmin { admin } => commands::set_admin(deps, info, admin),
        ExecuteMsg::SetFee {
            flash_loan_fee,
            treasury_fee,
            commission_fee,
        } => set_fee(deps, info, flash_loan_fee, treasury_fee, commission_fee),
        ExecuteMsg::AddToWhitelist { contract_addr } => commands::add_to_whitelist(deps, info, contract_addr),
        ExecuteMsg::RemoveFromWhitelist { contract_addr } => commands::remove_from_whitelist(deps, info, contract_addr),
        ExecuteMsg::FlashLoan { payload } => flashloan::handle_flashloan(deps, env, info, payload),
        ExecuteMsg::UpdateState {
            bluna_address,
            memory_address,
            whitelisted_contracts,
            allow_non_whitelisted,
            exchange_rate,
            total_bond_amount,
            last_index_modification,
            prev_vault_balance,
            actual_unbonded_amount,
            last_unbonded_time,
            last_processed_batch,
        } => commands::update_state(
            deps,
            info,
            bluna_address,
            memory_address,
            whitelisted_contracts,
            allow_non_whitelisted,
            exchange_rate,
            total_bond_amount,
            last_index_modification,
            prev_vault_balance,
            actual_unbonded_amount,
            last_unbonded_time,
            last_processed_batch,
        ),
        ExecuteMsg::UpdateParams {
            epoch_period,
            unbonding_period,
            peg_recovery_fee,
            er_threshold, } => commands::update_params(
            deps,
            env,
            info,
            epoch_period,
            unbonding_period,
            peg_recovery_fee,
            er_threshold,
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
    }
}
