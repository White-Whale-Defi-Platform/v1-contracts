use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps,
    DepsMut, Env, Fraction, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use protobuf::Message;
use semver::Version;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg;
use terraswap::querier::{query_balance, query_supply, query_token_balance};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use white_whale::anchor::{anchor_deposit_msg, anchor_withdraw_msg};
use white_whale::deposit_info::DepositInfo;
use white_whale::fee::{Fee, VaultFee};
use white_whale::memory::LIST_SIZE_LIMIT;
use white_whale::query::anchor::query_aust_exchange_rate;
use white_whale::tax::{compute_tax, into_msg_without_tax};
use white_whale::ust_vault::msg::*;
use white_whale::ust_vault::msg::{
    EstimateWithdrawFeeResponse, FeeResponse, ValueResponse, VaultQueryMsg as QueryMsg,
};

use crate::error::StableVaultError;
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{ProfitCheck, State, ADMIN, DEPOSIT_INFO, FEE, POOL_INFO, PROFIT, STATE};

const INSTANTIATE_REPLY_ID: u8 = 1u8;
pub const DEFAULT_LP_TOKEN_NAME: &str = "White Whale UST Vault LP Token";
pub const DEFAULT_LP_TOKEN_SYMBOL: &str = "wwVUst";
const ROUNDING_ERR_COMPENSATION: u32 = 10u32;

type VaultResult = Result<Response, StableVaultError>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:stablecoin-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> VaultResult {
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let state = State {
        anchor_money_market_address: deps.api.addr_validate(&msg.anchor_money_market_address)?,
        aust_address: deps.api.addr_validate(&msg.aust_address)?,
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
            treasury_fee: Fee {
                share: msg.treasury_fee,
            },
            commission_fee: Fee {
                share: msg.commission_fee,
            },
            treasury_addr: deps.api.addr_validate(&msg.treasury_addr)?,
        },
    )?;

    // Setup and save the relevant pools info in state. The saved pool will be the one used by the vault.
    let pool_info: &PoolInfoRaw = &PoolInfoRaw {
        contract_addr: env.contract.address.clone(),
        liquidity_token: Addr::unchecked(""),
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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity { asset } => try_provide_liquidity(deps, env, info, asset),
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
            treasury_fee,
            commission_fee,
        } => set_fee(deps, info, flash_loan_fee, treasury_fee, commission_fee),
        ExecuteMsg::AddToWhitelist { contract_addr } => add_to_whitelist(deps, info, contract_addr),
        ExecuteMsg::RemoveFromWhitelist { contract_addr } => {
            remove_from_whitelist(deps, info, contract_addr)
        }
        ExecuteMsg::FlashLoan { payload } => handle_flashloan(deps, env, info, payload),
        ExecuteMsg::UpdateState {
            anchor_money_market_address,
            aust_address,
            allow_non_whitelisted,
        } => update_state(
            deps,
            info,
            anchor_money_market_address,
            aust_address,
            allow_non_whitelisted,
        ),
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
        CallbackMsg::AfterTrade { loan_fee } => after_trade(deps, env, info, loan_fee),
    }
}

//----------------------------------------------------------------------------------------
//  EXECUTE FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

pub fn handle_flashloan(
    mut deps: DepsMut,
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
    if !whitelisted_contracts.contains(&deps.api.addr_validate(&info.sender.to_string())?) {
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
    let (total_value, stables_available, _) = compute_total_value(&env, deps.as_ref(), &pool_info)?;
    let requested_asset = payload.requested_asset;

    // Max tax buffer will be 2 transfers of the borrowed assets
    // Anchor -> Vault -> Caller
    let tax_buffer = Uint128::from(2u32) * requested_asset.compute_tax(&deps.querier)?
        + Uint128::from(ROUNDING_ERR_COMPENSATION);

    if total_value < requested_asset.amount + tax_buffer {
        return Err(StableVaultError::Broke {});
    }
    // Init response
    let mut response = Response::new().add_attribute("Action", "Flashloan");

    // Withdraw funds from Anchor if needed
    // FEE_BUFFER as buffer for fees and taxes
    if (requested_asset.amount + tax_buffer) > stables_available {
        // Attempt to remove some money from anchor
        let to_withdraw = (requested_asset.amount + tax_buffer) - stables_available;
        let aust_exchange_rate = query_aust_exchange_rate(
            env.clone(),
            deps.as_ref(),
            state.anchor_money_market_address.to_string(),
        )?;

        let withdraw_msg = anchor_withdraw_msg(
            state.aust_address,
            state.anchor_money_market_address,
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

    // Sets the current value of the vault and save logs
    response = response.add_attributes(before_trade(deps.branch(), env.clone())?);

    // Call encapsulate function
    encapsulate_payload(deps.as_ref(), env, response, loan_fee)
}

// This function should be called alongside a deposit of UST into the contract.
pub fn try_provide_liquidity(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    asset: Asset,
) -> VaultResult {
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    let profit = PROFIT.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let denom = deposit_info.clone().get_denom()?;

    if profit.last_balance != Uint128::zero() {
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
        compute_total_value(&env, deps.as_ref(), &info)?;
    // Get total supply of LP tokens and calculate share
    let total_share = query_supply(&deps.querier, info.liquidity_token.clone())?;

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
        contract_addr: info.liquidity_token.to_string(),
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
            state.anchor_money_market_address,
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
    let profit = PROFIT.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let denom = DEPOSIT_INFO.load(deps.storage)?.get_denom()?;
    let fee_config = FEE.load(deps.storage)?;

    if profit.last_balance != Uint128::zero() {
        return Err(StableVaultError::DepositDuringLoan {});
    }

    // Logging var
    let mut attrs = vec![];

    // Calculate share of pool and requested pool value
    let lp_addr = info.liquidity_token.clone();
    let total_share: Uint128 = query_supply(&deps.querier, lp_addr)?;
    let (total_value, _, uaust_value_in_contract) =
        compute_total_value(&env, deps.as_ref(), &info)?;
    // Get treasury fee in LP tokens
    let treasury_fee = get_treasury_fee(deps.as_ref(), amount)?;
    // Share with fee deducted.
    let share_ratio: Decimal = Decimal::from_ratio(amount - treasury_fee, total_share);
    let mut refund_amount: Uint128 = total_value * share_ratio;
    attrs.push(("Post-fee received:", refund_amount.to_string()));

    // Init response
    let mut response = Response::new();
    // Available aUST
    let max_aust_amount = query_token_balance(
        &deps.querier,
        state.aust_address.clone(),
        env.contract.address.clone(),
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
            env,
            deps.as_ref(),
            state.anchor_money_market_address.to_string(),
        )?;

        if uaust_value_in_contract < refund_amount {
            // Withdraw all aUST left
            let withdraw_msg = anchor_withdraw_msg(
                state.aust_address.clone(),
                state.anchor_money_market_address,
                max_aust_amount,
            )?;
            // Add msg to response and update withdrawn value
            response = response.add_message(withdraw_msg);
            withdrawn_ust.amount = uaust_value_in_contract;
        } else {
            // Repay user share of aUST
            let withdraw_amount = refund_amount * aust_exchange_rate.inv().unwrap();

            let withdraw_msg = anchor_withdraw_msg(
                state.aust_address,
                state.anchor_money_market_address,
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

    // LP token treasury Asset
    let lp_token_treasury_fee = Asset {
        info: AssetInfo::Token {
            contract_addr: info.liquidity_token.to_string(),
        },
        amount: treasury_fee,
    };

    // Construct treasury fee msg.
    let treasury_fee_msg = fee_config.treasury_fee.msg(
        deps.as_ref(),
        lp_token_treasury_fee,
        fee_config.treasury_addr,
    )?;
    attrs.push(("War chest fee:", treasury_fee.to_string()));

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
        contract_addr: info.liquidity_token.to_string(),
        // Burn exludes fee
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: (amount - treasury_fee),
        })?,
        funds: vec![],
    });

    Ok(response
        .add_message(refund_msg)
        .add_message(burn_msg)
        .add_message(treasury_fee_msg)
        .add_attribute("action:", "withdraw_liquidity")
        .add_attributes(attrs))
}

/// Sends the commission fee which is a function of the profit made by the contract, forwarded by the profit-check contract
fn send_commissions(deps: Deps, _info: MessageInfo, profit: Uint128) -> VaultResult {
    let fees = FEE.load(deps.storage)?;

    let commission_amount = fees.commission_fee.compute(profit);

    // Construct commission msg
    let refund_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        amount: commission_amount,
    };
    let commission_msg = refund_asset.into_msg(&deps.querier, fees.treasury_addr)?;

    Ok(Response::new()
        .add_attribute("treasury commission:", commission_amount.to_string())
        .add_message(commission_msg))
}

// Resets last trade and sets current UST balance of caller
pub fn before_trade(deps: DepsMut, env: Env) -> StdResult<Vec<(&str, String)>> {
    let mut conf = PROFIT.load(deps.storage)?;

    // last_balance call can not be reset until after the loan.
    if conf.last_balance != Uint128::zero() {
        return Err(StdError::generic_err(
            StableVaultError::Nonzero {}.to_string(),
        ));
    }

    conf.last_profit = Uint128::zero();

    // Index 0 = total_value
    conf.last_balance = total_value(deps.as_ref(), &env)?.0;
    PROFIT.save(deps.storage, &conf)?;

    Ok(vec![(
        "value before trade: ",
        conf.last_balance.to_string(),
    )])
}

// Checks if balance increased after the trade
pub fn after_trade(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    loan_fee: Uint128,
) -> VaultResult {
    // Deposit funds into anchor if applicable.
    let response = try_anchor_deposit(deps.branch(), env.clone())?;

    let mut conf = PROFIT.load(deps.storage)?;

    let balance = total_value(deps.as_ref(), &env)?.0;

    // Check if balance increased with expected fee, otherwise cancel everything
    if balance < conf.last_balance + loan_fee {
        return Err(StableVaultError::CancelLosingTrade {});
    }

    let profit = balance - conf.last_balance;

    conf.last_profit = profit;
    conf.last_balance = Uint128::zero();
    PROFIT.save(deps.storage, &conf)?;

    let commission_response = send_commissions(deps.as_ref(), info, profit)?;

    Ok(response
        // Send commission of profit to Treasury
        .add_submessages(commission_response.messages)
        .add_attributes(commission_response.attributes)
        .add_attribute("value after commission: ", balance.to_string()))
}

//----------------------------------------------------------------------------------------
//  HELPER FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

/// Helper method which encapsulates the requested funds.
/// This function prevents callers from doing unprofitable actions
/// with the vault funds and makes sure the funds are returned by
/// the borrower.
pub fn encapsulate_payload(
    _deps: Deps,
    env: Env,
    response: Response,
    loan_fee: Uint128,
) -> VaultResult {
    let total_response: Response = Response::new().add_attributes(response.attributes);

    // Callback for after the loan
    let after_trade = CallbackMsg::AfterTrade { loan_fee }.to_cosmos_msg(&env.contract.address)?;

    Ok(total_response
        // Add response that:
        // 1. Withdraws funds from Anchor if needed
        // 2. Sends funds to the borrower
        // 3. Calls the borrow contract through the provided callback msg
        .add_submessages(response.messages)
        // After borrower actions, deposit the received funds back into
        // Anchor if applicable
        // Call profit-check to cancel the borrow if
        // no profit is made.
        .add_message(after_trade))
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
            if deps.api.addr_validate(&msg_info.sender.to_string())? != info.liquidity_token {
                return Err(StableVaultError::Unauthorized {});
            }
            try_withdraw_liquidity(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
    }
}

/// compute total value of deposits in UST and return a tuple with those values.
/// (total, stable, aust)
pub fn compute_total_value(
    env: &Env,
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
        env.clone(),
        deps,
        state.anchor_money_market_address.to_string(),
    )?;
    let aust_value_in_ust = aust_exchange_rate * aust_amount;

    let total_deposits_in_ust = stable_amount + aust_value_in_ust;
    Ok((total_deposits_in_ust, stable_amount, aust_value_in_ust))
}

pub fn get_treasury_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let fee_config = FEE.load(deps.storage)?;
    let fee = fee_config.treasury_fee.compute(amount);
    Ok(fee)
}

pub fn get_withdraw_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let treasury_fee = get_treasury_fee(deps, amount)?;
    let anchor_withdraw_fee = compute_tax(
        deps,
        &Coin::new((amount - treasury_fee).u128(), String::from("uusd")),
    )?;
    let stable_transfer_fee = compute_tax(
        deps,
        &Coin::new(
            (amount - treasury_fee - anchor_withdraw_fee).u128(),
            String::from("uusd"),
        ),
    )?;
    // Two transfers (anchor -> vault -> user) so ~2x tax.
    Ok(treasury_fee + anchor_withdraw_fee + stable_transfer_fee)
}

//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

fn try_anchor_deposit(deps: DepsMut, env: Env) -> VaultResult {
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
            state.anchor_money_market_address,
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
            meta.liquidity_token = api.addr_validate(liquidity_token)?;
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
    allow_non_whitelisted: Option<bool>,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    let api = deps.api;

    if let Some(anchor_money_market_address) = anchor_money_market_address {
        state.anchor_money_market_address = api.addr_validate(&anchor_money_market_address)?;
    }

    if let Some(aust_address) = aust_address {
        state.aust_address = api.addr_validate(&aust_address)?;
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
        .contains(&deps.api.addr_validate(&contract_addr)?)
    {
        return Err(StableVaultError::AlreadyWhitelisted {});
    }

    // This is a limit to prevent potentially running out of gas when doing lookups on the whitelist
    if state.whitelisted_contracts.len() >= LIST_SIZE_LIMIT {
        return Err(StableVaultError::WhitelistLimitReached {});
    }

    // Add contract to whitelist.
    state
        .whitelisted_contracts
        .push(deps.api.addr_validate(&contract_addr)?);
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
        .contains(&deps.api.addr_validate(&contract_addr)?)
    {
        return Err(StableVaultError::NotWhitelisted {});
    }

    // Remove contract from whitelist.
    let canonical_addr = deps.api.addr_validate(&contract_addr)?;
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
    treasury_fee: Option<Fee>,
    commission_fee: Option<Fee>,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut fee_config = FEE.load(deps.storage)?;

    if let Some(fee) = flash_loan_fee {
        if fee.share >= Decimal::percent(100) {
            return Err(StableVaultError::InvalidFee {});
        }
        fee_config.flash_loan_fee = fee;
    }
    if let Some(fee) = treasury_fee {
        if fee.share >= Decimal::percent(100) {
            return Err(StableVaultError::InvalidFee {});
        }
        fee_config.treasury_fee = fee;
    }
    if let Some(fee) = commission_fee {
        if fee.share >= Decimal::percent(100) {
            return Err(StableVaultError::InvalidFee {});
        }
        fee_config.commission_fee = fee;
    }

    FEE.save(deps.storage, &fee_config)?;
    Ok(Response::default())
}

//----------------------------------------------------------------------------------------
//  QUERY HANDLERS
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PoolConfig {} => to_binary(&try_query_config(deps)?),
        QueryMsg::PoolState {} => to_binary(&try_query_pool_state(env, deps)?),
        QueryMsg::State {} => to_binary(&try_query_state(deps)?),
        QueryMsg::Fees {} => to_binary(&query_fees(deps)?),
        QueryMsg::VaultValue {} => to_binary(&query_total_value(env, deps)?),
        QueryMsg::EstimateWithdrawFee { amount } => {
            to_binary(&estimate_withdraw_fee(deps, amount)?)
        }
        QueryMsg::LastBalance {} => to_binary(&try_query_last_balance(deps)?),
        QueryMsg::LastProfit {} => to_binary(&try_query_last_profit(deps)?),
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

pub fn try_query_pool_state(env: Env, deps: Deps) -> StdResult<PoolResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let assets: [Asset; 2] = info.query_pools(deps, info.contract_addr.clone())?;
    let total_share: Uint128 = query_supply(&deps.querier, info.liquidity_token.clone())?;

    let (total_value_in_ust, _, _) = compute_total_value(&env, deps, &info)?;

    Ok(PoolResponse {
        assets,
        total_value_in_ust,
        total_share,
        liquidity_token: info.liquidity_token.into(),
    })
}

pub fn total_value(deps: Deps, env: &Env) -> StdResult<(Uint128, Uint128, Uint128)> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    compute_total_value(env, deps, &info)
}
pub fn query_total_value(env: Env, deps: Deps) -> StdResult<ValueResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (total_ust_value, _, _) = compute_total_value(&env, deps, &info)?;
    Ok(ValueResponse { total_ust_value })
}

pub fn try_query_last_profit(deps: Deps) -> StdResult<LastProfitResponse> {
    let conf = PROFIT.load(deps.storage)?;
    Ok(LastProfitResponse {
        last_profit: conf.last_profit,
    })
}

pub fn try_query_last_balance(deps: Deps) -> StdResult<LastBalanceResponse> {
    let conf = PROFIT.load(deps.storage)?;
    Ok(LastBalanceResponse {
        last_balance: conf.last_balance,
    })
}
