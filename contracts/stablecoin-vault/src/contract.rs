use cosmwasm_std::{
    entry_point, from_binary, to_binary, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Decimal,
    Deps, DepsMut, Env, Fraction, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use protobuf::Message;

use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg;
use terraswap::querier::{query_balance, query_supply, query_token_balance};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};

use white_whale::anchor::{anchor_deposit_msg, anchor_withdraw_msg};
use white_whale::deposit_info::DepositInfo;
use white_whale::fee::{Fee, VaultFee};
use white_whale::profit_check::msg::ExecuteMsg as ProfitCheckMsg;
use white_whale::profit_check::msg::LastBalanceResponse;
use white_whale::profit_check::msg::QueryMsg as ProfitCheckQueryMsg;
use white_whale::query::anchor::query_aust_exchange_rate;
use white_whale::ust_vault::msg::{
    EstimateWithdrawFeeResponse, FeeResponse, ValueResponse, VaultQueryMsg as QueryMsg,
};

use cw2::{get_contract_version, set_contract_version};
use semver::Version;
use white_whale::tax::{compute_tax, into_msg_without_tax};
use white_whale::ust_vault::msg::*;

use crate::error::StableVaultError;
use crate::pool_info::{PoolInfo, PoolInfoRaw};

use crate::response::MsgInstantiateContractResponse;
use crate::state::{State, ADMIN, DEPOSIT_INFO, FEE, POOL_INFO, STATE};

const FEE_BUFFER: u64 = 10_000_000u64;
const INSTANTIATE_REPLY_ID: u8 = 1u8;
pub const DEFAULT_LP_TOKEN_NAME: &str = "White Whale UST Vault LP Token";
pub const DEFAULT_LP_TOKEN_SYMBOL: &str = "wwVUst";

type VaultResult = Result<Response, StableVaultError>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:stablecoin-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> VaultResult {
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let state = State {
        anchor_money_market_address: deps
            .api
            .addr_canonicalize(&msg.anchor_money_market_address)?,
        aust_address: deps.api.addr_canonicalize(&msg.aust_address)?,
        profit_check_address: deps.api.addr_canonicalize(&msg.profit_check_address)?,
        whitelisted_contracts: vec![],
        allow_non_whitelisted: false,
    };

    // Store the initial config
    STATE.save(deps.storage, &state)?;
    DEPOSIT_INFO.save(
        deps.storage,
        &DepositInfo {
            asset_info: msg.asset_info.clone(),
        },
    )?;
    // Setup the fees system with a fee and other contract addresses
    FEE.save(
        deps.storage,
        &VaultFee {
            flash_loan_fee: Fee {
                share: msg.flash_loan_fee,
            },
            warchest_fee: Fee {
                share: msg.warchest_fee,
            },
            commission_fee: Fee {
                share: msg.commission_fee,
            },
            warchest_addr: deps.api.addr_canonicalize(&msg.warchest_addr)?,
        },
    )?;

    // Setup and save the relevant pools info in state. The saved pool will be the one used by the vault.
    let pool_info: &PoolInfoRaw = &PoolInfoRaw {
        contract_addr: env.contract.address.clone(),
        liquidity_token: CanonicalAddr::from(vec![]),
        stable_cap: msg.stable_cap,
        asset_infos: [
            msg.asset_info.to_raw(deps.api)?,
            AssetInfo::Token {
                contract_addr: msg.aust_address,
            }
            .to_raw(deps.api)?,
        ],
    };
    POOL_INFO.save(deps.storage, pool_info)?;
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
            label: "White Whale Stablecoin Vault LP".to_string(),
        }
        .into(),
        gas_limit: None,
        id: u64::from(INSTANTIATE_REPLY_ID),
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VaultResult {
    // let data = deps
    //     .storage
    //     .get(CONFIG_KEY)
    //     .ok_or_else(|| StdError::not_found("State"))?;
    // // We can start a new State object from the old one
    // let mut config: State = from_slice(&data)?;
    // // And use something provided in MigrateMsg to update the state of the migrated contract
    // config.verifier = deps.api.addr_validate(&msg.verifier)?;
    // // Then store our modified State
    // deps.storage.set(CONFIG_KEY, &to_vec(&config)?);
    // If we have no need to update the State of the contract then just Response::default() should suffice
    // in this case, the code is still updated, the migration does not change the contract addr or funds
    // if this is the case you desire, consider making the new Addr part of the MigrateMsg and then doing
    // a payout

    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // If state structure changed in any contract version in the way migration is needed, it
        // should occur here
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> VaultResult {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity { asset } => try_provide_liquidity(deps, info, asset),
        ExecuteMsg::SetStableCap { stable_cap } => set_stable_cap(deps, info, stable_cap),
        ExecuteMsg::SetAdmin { admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
            Ok(Response::default()
                .add_attribute("previous admin", previous_admin)
                .add_attribute("admin", admin))
        }
        ExecuteMsg::SetFee {
            flash_loan_fee,
            warchest_fee,
            commission_fee,
        } => set_fee(deps, info, flash_loan_fee, warchest_fee, commission_fee),
        ExecuteMsg::AddToWhitelist { contract_addr } => add_to_whitelist(deps, info, contract_addr),
        ExecuteMsg::RemoveFromWhitelist { contract_addr } => {
            remove_from_whitelist(deps, info, contract_addr)
        }
        ExecuteMsg::FlashLoan { payload } => handle_flashloan(deps, env, info, payload),
        ExecuteMsg::UpdateState {
            anchor_money_market_address,
            aust_address,
            profit_check_address,
            allow_non_whitelisted,
        } => update_state(
            deps,
            info,
            anchor_money_market_address,
            aust_address,
            profit_check_address,
            allow_non_whitelisted,
        ),
        ExecuteMsg::SendWarchestCommission { profit } => {
            send_commissions(deps.as_ref(), info, profit)
        }
        ExecuteMsg::Callback(msg) => _handle_callback(deps, env, info, msg),
    }
}

//----------------------------------------------------------------------------------------
//  PRIVATE FUNCTIONS
//----------------------------------------------------------------------------------------

fn _handle_callback(deps: DepsMut, env: Env, info: MessageInfo, msg: CallbackMsg) -> VaultResult {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(StableVaultError::NotCallback {});
    }
    match msg {
        CallbackMsg::AfterSuccessfulLoanCallback {} => after_successful_loan_callback(deps, env),
        // Possibility to add more callbacks in future.
    }
}

//----------------------------------------------------------------------------------------
//  EXECUTE FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

pub fn handle_flashloan(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payload: FlashLoanPayload,
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    let fees = FEE.load(deps.storage)?;
    let whitelisted_contracts = state.whitelisted_contracts;
    let whitelisted: bool;
    // Check if requested asset is base token of vault
    deposit_info.assert(&payload.requested_asset.info)?;

    // Check if sender is whitelisted
    if !whitelisted_contracts.contains(&deps.api.addr_canonicalize(&info.sender.to_string())?) {
        // Check if non-whitelisted are allowed to borrow
        if state.allow_non_whitelisted {
            whitelisted = false;
        } else {
            return Err(StableVaultError::NotWhitelisted {});
        }
    } else {
        whitelisted = true;
    }

    // Do we have enough funds?
    let pool_info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (total_value, stables_available, _) = compute_total_value(deps.as_ref(), &pool_info)?;
    let requested_asset = payload.requested_asset;

    if total_value < requested_asset.amount + Uint128::from(FEE_BUFFER) {
        return Err(StableVaultError::Broke {});
    }
    // Init response
    let mut response = Response::new().add_attribute("Action", "Flashloan");

    // Withdraw funds from Anchor if needed
    // FEE_BUFFER as buffer for fees and taxes
    if (requested_asset.amount + Uint128::from(FEE_BUFFER)) > stables_available {
        // Attempt to remove some money from anchor
        let to_withdraw = (requested_asset.amount + Uint128::from(FEE_BUFFER)) - stables_available;
        let aust_exchange_rate = query_aust_exchange_rate(
            deps.as_ref(),
            deps.api
                .addr_humanize(&state.anchor_money_market_address)?
                .to_string(),
        )?;

        let withdraw_msg = anchor_withdraw_msg(
            deps.api.addr_humanize(&state.aust_address)?,
            deps.api.addr_humanize(&state.anchor_money_market_address)?,
            to_withdraw * aust_exchange_rate.inv().unwrap(),
        )?;
        // Add msg to response and update withdrawn value
        response = response
            .add_message(withdraw_msg)
            .add_attribute("Anchor withdrawal", to_withdraw.to_string())
            .add_attribute("ust_aust_rate", aust_exchange_rate.to_string());
    }

    // If caller not whitelisted, calculate flashloan fee

    let loan_fee: Uint128 = if whitelisted {
        Uint128::zero()
    } else {
        fees.flash_loan_fee.compute(requested_asset.amount)
    };

    // Construct transfer of funds msg, tax is accounted for by buffer
    let loan_msg = into_msg_without_tax(requested_asset, info.sender.clone())?;
    response = response.add_message(loan_msg);

    // Construct return call with received binary
    let return_call = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.sender.into(),
        msg: payload.callback,
        funds: vec![],
    });

    response = response.add_message(return_call);

    // Call encapsulate function
    encapsulate_payload(deps.as_ref(), env, response, loan_fee)
}

// This function should be called alongside a deposit of UST into the contract.
pub fn try_provide_liquidity(deps: DepsMut, msg_info: MessageInfo, asset: Asset) -> VaultResult {
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let denom = deposit_info.clone().get_denom()?;

    // User is not able to deposit into the vault if he is using the flashloan
    let profit_check_response: LastBalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps
                .api
                .addr_humanize(&state.profit_check_address)?
                .to_string(),
            msg: to_binary(&ProfitCheckQueryMsg::LastBalance {})?,
        }))?;
    if profit_check_response.last_balance != Uint128::zero() {
        return Err(StableVaultError::DepositDuringLoan {});
    }

    // Init vector for logging
    let mut attrs = vec![];
    // Check if deposit matches claimed deposit.
    deposit_info.assert(&asset.info)?;
    asset.assert_sent_native_token_balance(&msg_info)?;
    attrs.push(("Action:", String::from("Deposit to vault")));
    attrs.push(("Received funds:", asset.to_string()));

    // Received deposit to vault
    let deposit: Uint128 = asset.amount;

    // Get total value in Vault
    let (total_deposits_in_ust, stables_in_contract, _) =
        compute_total_value(deps.as_ref(), &info)?;
    // Get total supply of LP tokens and calculate share
    let total_share = query_supply(
        &deps.querier,
        deps.api.addr_humanize(&info.liquidity_token)?,
    )?;

    let share = if total_share == Uint128::zero()
        || total_deposits_in_ust.checked_sub(deposit)? == Uint128::zero()
    {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_ust.checked_sub(deposit)?)
    };

    // mint LP token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: msg_info.sender.to_string(),
            amount: share,
        })?,
        funds: vec![],
    });

    let response = Response::new().add_attributes(attrs).add_message(msg);

    // If contract holds more then ANCHOR_DEPOSIT_THRESHOLD [UST] then try deposit to anchor and leave UST_CAP [UST] in contract.
    if stables_in_contract > info.stable_cap * Decimal::percent(150) {
        let deposit_amount = stables_in_contract - info.stable_cap;
        let anchor_deposit = Coin::new(deposit_amount.u128(), denom);
        let deposit_msg = anchor_deposit_msg(
            deps.as_ref(),
            deps.api.addr_humanize(&state.anchor_money_market_address)?,
            anchor_deposit,
        )?;
        return Ok(response.add_message(deposit_msg));
    };

    Ok(response)
}

/// Attempt to withdraw deposits. Fees are calculated and deducted in lp tokens.
/// This allowes the war-chest to accumulate a stake in the vault.
/// The refund is taken out of Anchor if possible.
/// Luna holdings are not eligible for withdrawal.
pub fn try_withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    sender: String,
    amount: Uint128,
) -> VaultResult {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let denom = DEPOSIT_INFO.load(deps.storage)?.get_denom()?;
    let fee_config = FEE.load(deps.storage)?;
    // User is not able to withdraw from the vault if he is using the flashloan
    let profit_check_response: LastBalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps
                .api
                .addr_humanize(&state.profit_check_address)?
                .to_string(),
            msg: to_binary(&ProfitCheckQueryMsg::LastBalance {})?,
        }))?;
    if profit_check_response.last_balance != Uint128::zero() {
        return Err(StableVaultError::DepositDuringLoan {});
    }

    // Logging var
    let mut attrs = vec![];

    // Calculate share of pool and requested pool value
    let lp_addr = deps.api.addr_humanize(&info.liquidity_token)?;
    let total_share: Uint128 = query_supply(&deps.querier, lp_addr)?;
    let (total_value, _, uaust_value_in_contract) = compute_total_value(deps.as_ref(), &info)?;
    // Get warchest fee in LP tokens
    let warchest_fee = get_warchest_fee(deps.as_ref(), amount)?;
    // Share with fee deducted.
    let share_ratio: Decimal = Decimal::from_ratio(amount - warchest_fee, total_share);
    let mut refund_amount: Uint128 = total_value * share_ratio;
    attrs.push(("Post-fee received:", refund_amount.to_string()));

    // Init response
    let mut response = Response::new();
    // Available aUST
    let max_aust_amount = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&state.aust_address)?,
        env.contract.address,
    )?;
    let mut withdrawn_ust = Asset {
        info: AssetInfo::NativeToken {
            denom: denom.clone(),
        },
        amount: Uint128::zero(),
    };

    // If we have aUST, try repay with that
    if max_aust_amount > Uint128::zero() {
        let aust_exchange_rate = query_aust_exchange_rate(
            deps.as_ref(),
            deps.api
                .addr_humanize(&state.anchor_money_market_address)?
                .to_string(),
        )?;

        if uaust_value_in_contract < refund_amount {
            // Withdraw all aUST left
            let withdraw_msg = anchor_withdraw_msg(
                deps.api.addr_humanize(&state.aust_address)?,
                deps.api.addr_humanize(&state.anchor_money_market_address)?,
                max_aust_amount,
            )?;
            // Add msg to response and update withdrawn value
            response = response.add_message(withdraw_msg);
            withdrawn_ust.amount = uaust_value_in_contract;
        } else {
            // Repay user share of aUST
            let withdraw_amount = refund_amount * aust_exchange_rate.inv().unwrap();

            let withdraw_msg = anchor_withdraw_msg(
                deps.api.addr_humanize(&state.aust_address)?,
                deps.api.addr_humanize(&state.anchor_money_market_address)?,
                withdraw_amount,
            )?;
            // Add msg to response and update withdrawn value
            response = response.add_message(withdraw_msg);
            withdrawn_ust.amount = refund_amount;
        };
        response = response
            .add_attribute("Max anchor withdrawal", max_aust_amount.to_string())
            .add_attribute("ust_aust_rate", aust_exchange_rate.to_string());

        // Compute tax on Anchor withdraw tx
        let withdrawtx_tax = withdrawn_ust.compute_tax(&deps.querier)?;
        refund_amount -= withdrawtx_tax;
        attrs.push(("After Anchor withdraw:", refund_amount.to_string()));
    };

    // LP token warchest Asset
    let lp_token_warchest_fee = Asset {
        info: AssetInfo::Token {
            contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        },
        amount: warchest_fee,
    };

    // Construct warchest fee msg.
    let warchest_fee_msg = fee_config.warchest_fee.msg(
        deps.as_ref(),
        lp_token_warchest_fee,
        deps.api.addr_humanize(&fee_config.warchest_addr)?,
    )?;
    attrs.push(("War chest fee:", warchest_fee.to_string()));

    // Construct refund message
    let refund_asset = Asset {
        info: AssetInfo::NativeToken { denom },
        amount: refund_amount,
    };
    let tax_assed = refund_asset.deduct_tax(&deps.querier)?;

    let refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: sender,
        amount: vec![tax_assed],
    });
    // LP burn msg
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        // Burn exludes fee
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: (amount - warchest_fee),
        })?,
        funds: vec![],
    });

    Ok(response
        .add_message(refund_msg)
        .add_message(burn_msg)
        .add_message(warchest_fee_msg)
        .add_attribute("action:", "withdraw_liquidity")
        .add_attributes(attrs))
}

/// Sends the commission fee which is a function of the profit made by the contract, forwarded by the profit-check contract
fn send_commissions(deps: Deps, info: MessageInfo, profit: Uint128) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    let fees = FEE.load(deps.storage)?;
    // Check if sender is profit check contract
    if deps.api.addr_humanize(&state.profit_check_address)? != info.sender {
        return Err(StableVaultError::Unauthorized {});
    }

    let commission_amount = fees.commission_fee.compute(profit);

    // Construct commission msg
    let refund_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        amount: commission_amount,
    };
    let commission_msg =
        refund_asset.into_msg(&deps.querier, deps.api.addr_humanize(&fees.warchest_addr)?)?;

    Ok(Response::new()
        .add_attribute("treasury commission:", commission_amount.to_string())
        .add_message(commission_msg))
}

//----------------------------------------------------------------------------------------
//  HELPER FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

/// Helper method which encapsulates the requested funds.
/// This function prevents callers from doing unprofitable actions
/// with the vault funds and makes sure the funds are returned by
/// the borrower.
pub fn encapsulate_payload(
    deps: Deps,
    env: Env,
    response: Response,
    loan_fee: Uint128,
) -> VaultResult {
    let state = STATE.load(deps.storage)?;

    let total_response: Response = Response::new();

    // Callback for after the loan
    let after_loan_msg =
        CallbackMsg::AfterSuccessfulLoanCallback {}.to_cosmos_msg(&env.contract.address)?;

    Ok(total_response
        // Call profit-check contract to store current value of funds
        // held in this contract
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&state.profit_check_address)?
                .to_string(),
            msg: to_binary(&ProfitCheckMsg::BeforeTrade {})?,
            funds: vec![],
        }))
        // Add response that:
        // 1. Withdraws funds from Anchor if needed
        // 2. Sends funds to the borrower
        // 3. Calls the borrow contract through the provided callback msg
        .add_submessages(response.messages)
        // After borrower actions, deposit the received funds back into
        // Anchor if applicable
        .add_message(after_loan_msg)
        // Call the profit-check again to cancel the borrow if
        // no profit is made.
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&state.profit_check_address)?
                .to_string(),
            msg: to_binary(&ProfitCheckMsg::AfterTrade { loan_fee })?,
            funds: vec![],
        })))
}

/// handler function invoked when the stablecoin-vault contract receives
/// a transaction. In this case it is triggered when the LP tokens are deposited
/// into the contract
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> VaultResult {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Swap { .. } => Err(StableVaultError::NoSwapAvailable {}),
        Cw20HookMsg::WithdrawLiquidity {} => {
            let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
            if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != info.liquidity_token {
                return Err(StableVaultError::Unauthorized {});
            }
            try_withdraw_liquidity(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
    }
}

// compute total value of deposits in UST and return a tuple with those values.
pub fn compute_total_value(
    deps: Deps,
    info: &PoolInfoRaw,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let state = STATE.load(deps.storage)?;
    let stable_info = info.asset_infos[0].to_normal(deps.api)?;
    let stable_denom = match stable_info {
        AssetInfo::Token { .. } => String::default(),
        AssetInfo::NativeToken { denom } => denom,
    };
    let stable_amount = query_balance(&deps.querier, info.contract_addr.clone(), stable_denom)?;

    let aust_info = info.asset_infos[1].to_normal(deps.api)?;
    let aust_amount = aust_info.query_pool(&deps.querier, deps.api, info.contract_addr.clone())?;
    let aust_exchange_rate = query_aust_exchange_rate(
        deps,
        deps.api
            .addr_humanize(&state.anchor_money_market_address)?
            .to_string(),
    )?;
    let aust_value_in_ust = aust_exchange_rate * aust_amount;

    let total_deposits_in_ust = stable_amount + aust_value_in_ust;
    Ok((total_deposits_in_ust, stable_amount, aust_value_in_ust))
}

pub fn get_warchest_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let fee_config = FEE.load(deps.storage)?;
    let fee = fee_config.warchest_fee.compute(amount);
    Ok(fee)
}

pub fn get_withdraw_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let warchest_fee = get_warchest_fee(deps, amount)?;
    let anchor_withdraw_fee = compute_tax(
        deps,
        &Coin::new((amount - warchest_fee).u128(), String::from("uusd")),
    )?;
    let stable_transfer_fee = compute_tax(
        deps,
        &Coin::new(
            (amount - warchest_fee - anchor_withdraw_fee).u128(),
            String::from("uusd"),
        ),
    )?;
    // Two transfers (anchor -> vault -> user) so ~2x tax.
    Ok(warchest_fee + anchor_withdraw_fee + stable_transfer_fee)
}

//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

fn after_successful_loan_callback(deps: DepsMut, env: Env) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    let stable_denom = DEPOSIT_INFO.load(deps.storage)?.get_denom()?;
    let stables_in_contract =
        query_balance(&deps.querier, env.contract.address, stable_denom.clone())?;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;

    // If contract holds more then ANCHOR_DEPOSIT_THRESHOLD [UST] then try deposit to anchor and leave UST_CAP [UST] in contract.
    if stables_in_contract > info.stable_cap * Decimal::percent(150) {
        let deposit_amount = stables_in_contract - info.stable_cap;
        let anchor_deposit = Coin::new(deposit_amount.u128(), stable_denom);
        let deposit_msg = anchor_deposit_msg(
            deps.as_ref(),
            deps.api.addr_humanize(&state.anchor_money_market_address)?,
            anchor_deposit,
        )?;

        return Ok(Response::new().add_message(deposit_msg));
    };
    Ok(Response::default())
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
            meta.liquidity_token = api.addr_canonicalize(liquidity_token)?;
            Ok(meta)
        })?;

        return Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token));
    }
    Ok(Response::default())
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

pub fn update_state(
    deps: DepsMut,
    info: MessageInfo,
    anchor_money_market_address: Option<String>,
    aust_address: Option<String>,
    profit_check_address: Option<String>,
    allow_non_whitelisted: Option<bool>,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    let api = deps.api;

    if let Some(anchor_money_market_address) = anchor_money_market_address {
        state.anchor_money_market_address = api.addr_canonicalize(&anchor_money_market_address)?;
    }

    if let Some(aust_address) = aust_address {
        state.aust_address = api.addr_canonicalize(&aust_address)?;
    }

    if let Some(profit_check_address) = profit_check_address {
        state.profit_check_address = api.addr_canonicalize(&profit_check_address)?;
    }

    if let Some(allow_non_whitelisted) = allow_non_whitelisted {
        state.allow_non_whitelisted = allow_non_whitelisted;
    }

    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("Update:", "Successfull"))
}

pub fn set_stable_cap(deps: DepsMut, msg_info: MessageInfo, stable_cap: Uint128) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let previous_cap = info.stable_cap;
    info.stable_cap = stable_cap;
    POOL_INFO.save(deps.storage, &info)?;
    Ok(Response::new()
        .add_attribute("new stable cap", stable_cap.to_string())
        .add_attribute("previous stable cap", previous_cap.to_string()))
}

pub fn add_to_whitelist(
    deps: DepsMut,
    msg_info: MessageInfo,
    contract_addr: String,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    // Check if contract is already in whitelist
    if state
        .whitelisted_contracts
        .contains(&deps.api.addr_canonicalize(&contract_addr)?)
    {
        return Err(StableVaultError::AlreadyWhitelisted {});
    }

    // Add contract to whitelist.
    state
        .whitelisted_contracts
        .push(deps.api.addr_canonicalize(&contract_addr)?);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Added contract to whitelist: ", contract_addr))
}

pub fn remove_from_whitelist(
    deps: DepsMut,
    msg_info: MessageInfo,
    contract_addr: String,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    // Check if contract is in whitelist
    if !state
        .whitelisted_contracts
        .contains(&deps.api.addr_canonicalize(&contract_addr)?)
    {
        return Err(StableVaultError::NotWhitelisted {});
    }

    // Remove contract from whitelist.
    let canonical_addr = deps.api.addr_canonicalize(&contract_addr)?;
    state
        .whitelisted_contracts
        .retain(|addr| *addr != canonical_addr);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Removed contract from whitelist: ", contract_addr))
}

pub fn set_fee(
    deps: DepsMut,
    msg_info: MessageInfo,
    flash_loan_fee: Option<Fee>,
    warchest_fee: Option<Fee>,
    commission_fee: Option<Fee>,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut fee_config = FEE.load(deps.storage)?;

    if let Some(fee) = flash_loan_fee {
        fee_config.flash_loan_fee = fee;
    }
    if let Some(fee) = warchest_fee {
        fee_config.warchest_fee = fee;
    }
    if let Some(fee) = commission_fee {
        fee_config.commission_fee = fee;
    }
    
    FEE.save(deps.storage, &fee_config)?;
    Ok(Response::default())
}

//----------------------------------------------------------------------------------------
//  QUERY HANDLERS
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PoolConfig {} => to_binary(&try_query_config(deps)?),
        QueryMsg::PoolState {} => to_binary(&try_query_pool_state(deps)?),
        QueryMsg::State {} => to_binary(&try_query_state(deps)?),
        QueryMsg::Fees {} => to_binary(&query_fees(deps)?),
        QueryMsg::VaultValue {} => to_binary(&query_total_value(deps)?),
        QueryMsg::EstimateWithdrawFee { amount } => {
            to_binary(&estimate_withdraw_fee(deps, amount)?)
        }
    }
}

pub fn query_fees(deps: Deps) -> StdResult<FeeResponse> {
    Ok(FeeResponse {
        fees: FEE.load(deps.storage)?,
    })
}

// amount in UST. Equal to the value of the offered LP tokens
pub fn estimate_withdraw_fee(
    deps: Deps,
    amount: Uint128,
) -> StdResult<EstimateWithdrawFeeResponse> {
    let fee = get_withdraw_fee(deps, amount)?;
    Ok(EstimateWithdrawFeeResponse {
        fee: vec![Coin {
            denom: DEPOSIT_INFO.load(deps.storage)?.get_denom()?,
            amount: fee,
        }],
    })
}

pub fn try_query_config(deps: Deps) -> StdResult<PoolInfo> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;

    info.to_normal(deps)
}
pub fn try_query_state(deps: Deps) -> StdResult<State> {
    STATE.load(deps.storage)
}

pub fn try_query_pool_state(deps: Deps) -> StdResult<PoolResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let assets: [Asset; 2] = info.query_pools(deps, info.contract_addr.clone())?;
    let total_share: Uint128 = query_supply(
        &deps.querier,
        deps.api.addr_humanize(&info.liquidity_token)?,
    )?;

    let (total_value_in_ust, _, _) = compute_total_value(deps, &info)?;

    Ok(PoolResponse {
        assets,
        total_value_in_ust,
        total_share,
    })
}

pub fn query_total_value(deps: Deps) -> StdResult<ValueResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (total_ust_value, _, _) = compute_total_value(deps, &info)?;
    Ok(ValueResponse { total_ust_value })
}
