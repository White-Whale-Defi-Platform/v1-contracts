use std::os::macos::raw::stat;

use cosmwasm_std::{
    attr, coins, from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, ReplyOn, Response, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::querier::query_supply;

use white_whale::denom::LUNA_DENOM;
use white_whale::fee::Fee;
use white_whale::luna_vault::luna_unbond_handler::msg::Cw20HookMsg::Unbond as UnbondHandlerUnbondMsg;
use white_whale::luna_vault::luna_unbond_handler::msg::InstantiateMsg;
use white_whale::luna_vault::msg::{Cw20HookMsg, UnbondHandlerMsg};
use white_whale::memory::LIST_SIZE_LIMIT;

use crate::contract::{VaultResult, INSTANTIATE_UNBOND_HANDLER_REPLY_ID};
use crate::error::LunaVaultError;
use crate::helpers::{check_fee, compute_total_value, get_lp_token_address, get_treasury_fee};
use crate::pool_info::PoolInfoRaw;
use crate::state::{
    deprecate_unbond_batches, get_deprecated_unbond_batch_ids, get_withdrawable_amount,
    get_withdrawable_unbond_batch_ids, prepare_next_unbond_batch, remove_unbond_wait_list,
    store_unbond_history, store_unbond_wait_list, UnbondHistory, ADMIN, CURRENT_BATCH,
    DEPOSIT_INFO, FEE, POOL_INFO, PROFIT, STATE, UNBOND_HANDLERS_ASSIGNED,
    UNBOND_HANDLERS_AVAILABLE,
};

/// handler function invoked when the luna-vault contract receives
/// a transaction. In this case it is triggered when the LP tokens are deposited
/// into the contract
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> VaultResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Unbond {} => {
            // only vLuna token contract can execute this message
            let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
            if deps.api.addr_validate(&msg_info.sender.to_string())? != info.liquidity_token {
                return Err(LunaVaultError::Unauthorized {});
            }
            unbond(deps, env, cw20_msg.amount, cw20_msg.sender)
        }
    }
}

// Deposits Luna into the contract.
pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    asset: Asset,
) -> VaultResult<Response> {
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    let profit = PROFIT.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;

    if profit.last_balance != Uint128::zero() {
        return Err(LunaVaultError::DepositDuringLoan {});
    }

    // Init vector for logging
    let mut attrs = vec![];
    // Check if deposit matches claimed deposit.
    deposit_info.assert(&asset.info)?;
    asset.assert_sent_native_token_balance(&msg_info)?;
    attrs.push(("action", String::from("provide_liquidity")));
    attrs.push(("received funds", asset.to_string()));

    // Received deposit to vault
    let deposit: Uint128 = asset.amount;

    // Get total value in Vault
    let (total_deposits_in_luna, _, _, _, _) = compute_total_value(&env, deps.as_ref(), &info)?;
    // Get total supply of vLuna tokens and calculate share
    let total_share = query_supply(&deps.querier, info.liquidity_token.clone())?;

    let share = if total_share == Uint128::zero()
        || total_deposits_in_luna.checked_sub(deposit)? == Uint128::zero()
    {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_luna.checked_sub(deposit)?)
    };

    // mint LP token to sender
    let mint_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: msg_info.sender.to_string(),
            amount: share,
        })?,
        funds: vec![],
    });

    let response = Response::new().add_attributes(attrs).add_message(mint_msg);
    // Deposit liquid luna into passive strategy
    deposit_passive_strategy(
        &deps.as_ref(),
        deposit,
        state.bluna_address,
        &state.astro_lp_address,
        response,
    )
}

// Deposits Luna into the passive strategy (Astroport) -> luna-bluna LP
pub(crate) fn deposit_passive_strategy(
    deps: &Deps,
    deposit_amount: Uint128,
    bluna_address: Addr,
    astro_lp_address: &Addr,
    response: Response,
) -> VaultResult<Response> {
    // split luna into half so half goes to purchase bLuna, remaining half is used as liquidity
    let luna_asset = astroport::asset::Asset {
        amount: deposit_amount.checked_div(Uint128::from(2_u8))?,
        info: astroport::asset::AssetInfo::NativeToken {
            denom: LUNA_DENOM.to_string(),
        },
    };

    // simulate the luna swap so we know the bluna return amount when we later provide liquidity
    let bluna_return: astroport::pair::SimulationResponse = deps.querier.query_wasm_smart(
        astro_lp_address,
        &astroport::pair::QueryMsg::Simulation {
            offer_asset: luna_asset.clone(),
        },
    )?;

    let bluna_asset = astroport::asset::Asset {
        amount: bluna_return.return_amount,
        info: astroport::asset::AssetInfo::Token {
            contract_addr: bluna_address,
        },
    };

    let bluna_purchase_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: astro_lp_address.to_string(),
        msg: to_binary(&astroport::pair::ExecuteMsg::Swap {
            offer_asset: luna_asset.clone(),
            belief_price: None,
            max_spread: None,
            to: None,
        })?,
        funds: vec![],
    });

    let deposit_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: astro_lp_address.to_string(),
        msg: to_binary(&astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: [luna_asset, bluna_asset],
            slippage_tolerance: None,
            auto_stake: None,
            receiver: None,
        })?,
        funds: vec![],
    });

    let response = response.add_messages(vec![
        bluna_purchase_msg, // 1. purchase bluna
        deposit_msg,        // 2. deposit bLuna/Luna to the LP as liquidity
    ]);

    Ok(response)
}

// Withdraws Luna from the passive strategy (Astroport): luna-bluna LP -> Luna + bLuna
pub(crate) fn withdraw_passive_strategy(
    _deps: &Deps,
    withdraw_amount: Uint128,
    _bluna_address: Addr,
    astro_lp_token_address: &Addr,
    astro_lp_address: &Addr,
    response: Response,
) -> VaultResult<Response> {
    // Msg that gets called on the pair address.
    let withdraw_msg: Binary = to_binary(&astroport::pair::Cw20HookMsg::WithdrawLiquidity {})?;

    // cw20 send message that transfers the LP tokens to the pair address
    let cw20_msg = Cw20ExecuteMsg::Send {
        contract: astro_lp_address.clone().into_string(),
        amount: withdraw_amount,
        msg: withdraw_msg,
    };

    // Call on LP token.
    let withdraw_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: String::from(astro_lp_token_address),
        msg: to_binary(&cw20_msg)?,
        funds: vec![],
    });

    // Leaving this here for now but commented, this logic allows us to offer luna or bLuna if caller is willing to assume fees
    // let bluna_asset = astroport::asset::Asset {
    //     amount: bluna_return.return_amount,
    //     info: astroport::asset::AssetInfo::Token {
    //         contract_addr: bluna_address,
    //     },
    // };
    //
    // let bluna_purchase_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: astro_lp_address.to_string(),
    //     msg: to_binary(&astroport::pair::ExecuteMsg::Swap {
    //         offer_asset: luna_asset.clone(),
    //         belief_price: None,
    //         max_spread: None,
    //         to: None,
    //     })?,
    //     funds: vec![],
    // });

    let response = response.add_messages(vec![
        withdraw_msg, // 1. withdraw bluna and Luna from LP.
                      // deposit_msg,        // 2-N. Further steps could include, swapping to another luna variant to have one token rather than 2.
    ]);

    Ok(response)
}

/// This message must be called by receive_cw20
/// This message will trigger the withdrawal waiting time and burn vluna token
fn unbond(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
    sender: String, // human who sent the vluna to us
) -> VaultResult<Response> {
    let profit = PROFIT.load(deps.storage)?;
    if profit.last_balance != Uint128::zero() {
        return Err(LunaVaultError::DepositDuringLoan {});
    }

    // Logging var
    let mut response = Response::new().add_attribute("action", "unbond");
    let mut attrs = vec![];
    attrs.push(("from", sender.clone()));
    attrs.push(("burnt_amount", amount.to_string()));

    // Get treasury fee in LP tokens
    let treasury_fee = get_treasury_fee(deps.as_ref(), amount)?;
    attrs.push(("treasury_fee", treasury_fee.to_string()));

    // Calculate share of pool and requested pool value
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let total_share = query_supply(&deps.querier, info.liquidity_token.clone())?;
    let (total_value_in_luna, _, _, _, _) = compute_total_value(&env, deps.as_ref(), &info)?;
    // Share with fee deducted.
    let share_ratio: Decimal = Decimal::from_ratio(amount - treasury_fee, total_share);
    let refund_amount: Uint128 = total_value_in_luna * share_ratio;
    attrs.push(("post_fee_unbonded_amount", refund_amount.to_string()));

    let current_batch = CURRENT_BATCH.load(deps.storage)?;

    // Add unbond to the wait list
    let sender_addr = deps.api.addr_validate(&sender)?;
    store_unbond_wait_list(deps.storage, current_batch.id, &sender_addr, refund_amount)?;

    let unbond_history = UnbondHistory {
        batch_id: current_batch.id,
        time: env.block.time.seconds(),
        amount: refund_amount,
        released: false,
    };
    store_unbond_history(deps.storage, current_batch.id, unbond_history)?;

    // Prepare for next unbond batch
    prepare_next_unbond_batch(deps.storage)?;

    // LP token treasury Asset
    let lp_token_treasury_fee = Asset {
        info: AssetInfo::Token {
            contract_addr: info.liquidity_token.to_string(),
        },
        amount: treasury_fee,
    };

    // Construct treasury fee msg.
    let fee_config = FEE.load(deps.storage)?;
    let treasury_fee_msg = fee_config.treasury_fee.msg(
        deps.as_ref(),
        lp_token_treasury_fee,
        fee_config.treasury_addr,
    )?;

    // Send Burn message to vluna contract
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.liquidity_token.to_string(),
        // Burn excludes treasury fee
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: amount - treasury_fee,
        })?,
        funds: vec![],
    });

    //------------------------------------------------
    //todo withdraw shares from LP
    let bluna_amount = Uint128::zero();

    // Check if there's a handler assigned to the user
    let unbond_handler_result = UNBOND_HANDLERS_ASSIGNED.load(deps.storage, &sender_addr);
    let unbond_handler: Addr;
    if unbond_handler_result.is_ok() {
        // there's an unbond handler assigned to the user
        unbond_handler = unbond_handler_result?.clone();

        // send bluna to unbond handler
        response =
            unbond_bluna_with_handler(deps, &response, bluna_amount.clone(), &unbond_handler);
    } else {
        // there's no unbond handlers assigned to the user
        // check if there are handlers available
        let unbond_handlers_available = UNBOND_HANDLERS_AVAILABLE.load(deps.storage)?;
        if unbond_handlers_available.is_empty() {
            // create a new handler if there are no handlers available
            let state = STATE.load(deps.storage)?;
            let unbond_handler_instantiation_msg = SubMsg {
                id: u64::from(INSTANTIATE_UNBOND_HANDLER_REPLY_ID),
                msg: WasmMsg::Instantiate {
                    admin: Some(env.contract.address.to_string()),
                    code_id: state.unbond_handler_code_id,
                    msg: to_binary(&InstantiateMsg {
                        owner: Some(sender_addr.to_string()),
                        memory_contract: state.memory_address.to_string(),
                    })?,
                    funds: vec![],
                    label: "White Whale Unbond Handler".to_string(),
                }
                .into(),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            };
            response = response.add_submessage(unbond_handler_instantiation_msg);
        } else {
            //otherwise, use an available one
            unbond_handler = unbond_handlers_available.first()?.clone();
            UNBOND_HANDLERS_ASSIGNED.save(deps.storage, &sender_addr, &&unbond_handler)?;

            // send bluna to unbond handler
            response =
                unbond_bluna_with_handler(deps, &response, bluna_amount.clone(), &unbond_handler);
        };
    }

    Ok(response
        .add_messages(vec![burn_msg, treasury_fee_msg])
        .add_attributes(attrs))
}

fn unbond_bluna_with_handler(
    deps: DepsMut,
    response: &Response,
    bluna_amount: Uint128,
    unbond_handler: &Addr,
) -> Response {
    let state = STATE.load(deps.storage)?;
    let unbond_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.bluna_address.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: unbond_handler.to_string(),
            amount: bluna_amount,
            msg: to_binary(&UnbondHandlerUnbondMsg {})?,
        })?,
        funds: vec![],
    });
    let response = response.add_message(unbond_msg);
    response
}

/// Withdraws unbonded luna after unbond has been called and the time lock period expired
pub fn withdraw_unbonded(deps: DepsMut, env: Env, msg_info: MessageInfo) -> VaultResult<Response> {
    let state = STATE.load(deps.storage)?;
    let withdrawable_time = env.block.time.seconds() - state.unbonding_period;
    let withdraw_amount =
        get_withdrawable_amount(deps.storage, &msg_info.sender, withdrawable_time)?;
    if withdraw_amount.is_zero() {
        return Err(LunaVaultError::NoWithdrawableAssetsAvailable(
            LUNA_DENOM.to_string(),
        ));
    }

    // remove batches to be withdrawn for the user
    let withdrawable_batch_ids =
        get_withdrawable_unbond_batch_ids(deps.storage, &msg_info.sender, withdrawable_time)?;
    deprecate_unbond_batches(deps.storage, withdrawable_batch_ids)?;
    let deprecated_batch_ids = get_deprecated_unbond_batch_ids(deps.storage, &msg_info.sender)?;
    remove_unbond_wait_list(deps.storage, deprecated_batch_ids, &msg_info.sender)?;

    // Send the money to the user
    let withdraw_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: msg_info.sender.to_string(),
        amount: coins(withdraw_amount.u128(), &*LUNA_DENOM.to_string()),
    });

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "withdraw_unbonded"),
            attr("from", env.contract.address),
            attr("amount", withdraw_amount),
        ])
        .add_message(withdraw_msg))
}

/// Sets a new admin
pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> VaultResult<Response> {
    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

/// Sets new fees for vault, flashloan and treasury
pub fn set_fee(
    deps: DepsMut,
    msg_info: MessageInfo,
    flash_loan_fee: Option<Fee>,
    treasury_fee: Option<Fee>,
    commission_fee: Option<Fee>,
) -> VaultResult<Response> {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut fee_config = FEE.load(deps.storage)?;

    if let Some(fee) = flash_loan_fee {
        fee_config.flash_loan_fee = check_fee(fee)?;
    }
    if let Some(fee) = treasury_fee {
        fee_config.treasury_fee = check_fee(fee)?;
    }
    if let Some(fee) = commission_fee {
        fee_config.commission_fee = check_fee(fee)?;
    }

    FEE.save(deps.storage, &fee_config)?;
    Ok(Response::default())
}

/// Adds a contract to the whitelist
pub fn add_to_whitelist(
    deps: DepsMut,
    msg_info: MessageInfo,
    contract_addr: String,
) -> VaultResult<Response> {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    // Check if contract is already in whitelist
    if state
        .whitelisted_contracts
        .contains(&deps.api.addr_validate(&contract_addr)?)
    {
        return Err(LunaVaultError::AlreadyWhitelisted {});
    }

    // This is a limit to prevent potentially running out of gas when doing lookups on the whitelist
    if state.whitelisted_contracts.len() >= LIST_SIZE_LIMIT {
        return Err(LunaVaultError::WhitelistLimitReached {});
    }

    // Add contract to whitelist.
    state
        .whitelisted_contracts
        .push(deps.api.addr_validate(&contract_addr)?);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Added contract to whitelist: ", contract_addr))
}

/// Removes a contract from the whitelist
pub fn remove_from_whitelist(
    deps: DepsMut,
    msg_info: MessageInfo,
    contract_addr: String,
) -> VaultResult<Response> {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    // Check if contract is in whitelist
    if !state
        .whitelisted_contracts
        .contains(&deps.api.addr_validate(&contract_addr)?)
    {
        return Err(LunaVaultError::NotWhitelisted {});
    }

    // Remove contract from whitelist.
    let contract_validated_addr = deps.api.addr_validate(&contract_addr)?;
    state
        .whitelisted_contracts
        .retain(|addr| *addr != contract_validated_addr);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Removed contract from whitelist: ", contract_addr))
}

/// Updates the contract state
pub fn update_state(
    deps: DepsMut,
    info: MessageInfo,
    bluna_address: Option<String>,
    astro_lp_address: Option<String>,
    memory_address: Option<String>,
    whitelisted_contracts: Option<Vec<String>>,
    allow_non_whitelisted: Option<bool>,
    unbonding_period: Option<u64>,
) -> VaultResult<Response> {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    let api = deps.api;

    let mut attrs = vec![];

    if let Some(bluna_address) = bluna_address {
        state.bluna_address = api.addr_validate(&bluna_address)?;
        attrs.push(("new bluna_address", bluna_address));
    }
    if let Some(astro_lp_address) = astro_lp_address {
        state.astro_lp_address = api.addr_validate(&astro_lp_address)?;
        attrs.push(("new astro_lp_address", astro_lp_address));
    }
    if let Some(memory_address) = memory_address {
        state.memory_address = api.addr_validate(&memory_address)?;
        attrs.push(("new memory_address", memory_address));
    }
    if let Some(whitelisted_contracts) = whitelisted_contracts {
        let mut contracts = vec![];
        for contract_addr in whitelisted_contracts.clone() {
            contracts.push(deps.api.addr_validate(&contract_addr)?);
        }
        state.whitelisted_contracts = contracts;
        attrs.push((
            "new whitelisted_contracts",
            format!("{:?}", whitelisted_contracts),
        ));
    }
    if let Some(allow_non_whitelisted) = allow_non_whitelisted {
        state.allow_non_whitelisted = allow_non_whitelisted;
        attrs.push((
            "new allow_non_whitelisted",
            allow_non_whitelisted.to_string(),
        ));
    }

    if let Some(unbonding_period) = unbonding_period {
        state.unbonding_period = unbonding_period;
        attrs.push(("new unbonding_period", unbonding_period.to_string()));
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(attrs))
}

pub fn swap_rewards(deps: DepsMut, env: Env, msg_info: MessageInfo) -> VaultResult<Response> {
    let state = STATE.load(deps.storage)?;
    // Check if sender is in whitelist, i.e. bot or bot proxy
    if !state.whitelisted_contracts.contains(&msg_info.sender) {
        return Err(LunaVaultError::NotWhitelisted {});
    }

    let mut response = Response::new();

    let passive_lp_token_address =
        get_lp_token_address(&deps.as_ref(), state.astro_lp_address.clone())?;

    // swap ASTRO rewards for Luna to stay in the vault
    let pending_tokens: astroport::generator::PendingTokenResponse =
        deps.querier.query_wasm_smart(
            state.astro_lp_address.clone(),
            &astroport::generator::QueryMsg::PendingToken {
                lp_token: passive_lp_token_address.clone().into_string(),
                user: env.contract.address.clone().into_string(),
            },
        )?;

    // get generator address
    let astro_factory_config: astroport::factory::ConfigResponse = deps.querier.query_wasm_smart(
        state.astro_factory_address.clone(),
        &astroport::factory::QueryMsg::Config {},
    )?;
    let astro_generator_address = astro_factory_config.generator_address.ok_or_else(|| {
        LunaVaultError::generic_err("Astroport generator was not set in factory config")
    })?;

    // get ASTRO token address
    let astro_generator_config: astroport::generator::ConfigResponse =
        deps.querier.query_wasm_smart(
            astro_generator_address.clone(),
            &astroport::generator::QueryMsg::Config {},
        )?;
    let astro_token_address = astro_generator_config.astro_token;
    let astro_pending = astroport::asset::Asset {
        amount: pending_tokens.pending,
        info: astroport::asset::AssetInfo::Token {
            contract_addr: astro_token_address.clone(),
        },
    };

    // withdraw ASTRO rewards
    let withdraw_rewards_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: astro_generator_address.into_string(),
        msg: to_binary(&astroport::generator::ExecuteMsg::ClaimRewards {
            lp_tokens: vec![passive_lp_token_address.into_string()],
        })?,
        funds: vec![],
    }
    .into();

    // swap ASTRO into Luna
    // first, get the address of the pool from Astroport
    // then, perform the swap, and finally perform passive strategy with the gained luna
    let astro_luna_pool_address: astroport::asset::PairInfo = deps.querier.query_wasm_smart(
        state.astro_factory_address,
        &astroport::factory::QueryMsg::Pair {
            asset_infos: [
                astroport::asset::AssetInfo::Token {
                    contract_addr: astro_token_address,
                },
                astroport::asset::AssetInfo::NativeToken {
                    denom: LUNA_DENOM.to_string(),
                },
            ],
        },
    )?;

    let swap_simulation_response = astroport::querier::simulate(
        &deps.querier,
        astro_luna_pool_address.contract_addr.clone(),
        &astro_pending,
    )?;
    let swap_luna_return = swap_simulation_response.return_amount;

    let swap_astro_message = WasmMsg::Execute {
        contract_addr: astro_luna_pool_address.contract_addr.into_string(),
        msg: to_binary(&astroport::pair::ExecuteMsg::Swap {
            offer_asset: astro_pending.clone(),
            belief_price: None,
            max_spread: None,
            to: None,
        })?,
        funds: vec![],
    }
    .into();

    response = response.add_messages([withdraw_rewards_msg, swap_astro_message]);

    // Deposit luna into passive strategy
    response = deposit_passive_strategy(
        &deps.as_ref(),
        swap_luna_return,
        state.bluna_address,
        &state.astro_lp_address,
        response,
    )?;

    Ok(response.add_attributes(vec![
        attr("action", "swap_rewards"),
        attr("astro_swapped", astro_pending.amount),
        attr("luna_return", swap_luna_return),
    ]))
}

pub(crate) fn handle_unbond_handler_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UnbondHandlerMsg,
) -> VaultResult<Response> {
    todo!()
    //todo verify the message was sent by a handler, i.e. check address with the list of unbonded handlers
    // registered by the vault
    //todo find the unbond handler address on the map and clear it, place it back to the
    // vector of available unbond handlers
}
