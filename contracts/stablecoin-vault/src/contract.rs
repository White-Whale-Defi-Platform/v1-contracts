use cosmwasm_std::{ entry_point, CanonicalAddr,
    from_binary, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, WasmMsg, Uint128, Decimal, SubMsg, Reply, ReplyOn
};
use terra_cosmwasm::{create_swap_msg, TerraMsgWrapper};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg;
use terraswap::querier::{query_balance, query_token_balance, query_supply};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};

use white_whale::burn::msg::ExecuteMsg as BurnMsg;
use white_whale::denom::LUNA_DENOM;
use white_whale::deposit_info::DepositInfo;
use white_whale::fee::{Fee, CappedFee, VaultFee};
use white_whale::msg::{create_terraswap_msg, VaultQueryMsg as QueryMsg, AnchorMsg};
use white_whale::query::terraswap::simulate_swap as simulate_terraswap_swap;
use white_whale::query::anchor::query_aust_exchange_rate;
use white_whale::profit_check::msg::{HandleMsg as ProfitCheckMsg};
use white_whale::anchor::try_deposit_to_anchor as try_deposit;

use crate::error::StableVaultError;
use crate::msg::{ExecuteMsg, InitMsg, PoolResponse, DepositResponse};
use crate::state::{State, ADMIN, STATE, POOL_INFO, DEPOSIT_INFO, FEE, DEPOSIT_MANAGER};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::querier::{query_market_price, from_micro};
use crate::response::MsgInstantiateContractResponse;
use std::str::FromStr;


const INSTANTIATE_REPLY_ID: u64 = 1;

type VaultResult = Result<Response<TerraMsgWrapper>, StableVaultError>;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let state = State {
        trader: deps.api.addr_canonicalize(info.sender.as_str())?,
        pool_address: deps.api.addr_canonicalize(&msg.pool_address)?,
        anchor_money_market_address: deps.api.addr_canonicalize(&msg.anchor_money_market_address)?,
        aust_address: deps.api.addr_canonicalize(&msg.aust_address)?,
        seignorage_address: deps.api.addr_canonicalize(&msg.seignorage_address)?,
        profit_check_address: deps.api.addr_canonicalize(&msg.profit_check_address)?,
    };

    STATE.save(deps.storage, &state)?;
    DEPOSIT_INFO.save(deps.storage, &DepositInfo{
        asset_info: AssetInfo::NativeToken{ denom: msg.denom }
    })?;
    FEE.save(deps.storage, &VaultFee{
        burn_fee: CappedFee{
            fee: Fee{ share: msg.burn_vault_fee },
            max_fee: msg.max_burn_vault_fee
        },
        warchest_fee: Fee{ share: msg.warchest_fee },
        burn_addr: deps.api.addr_canonicalize(&msg.burn_addr)?,
        warchest_addr: deps.api.addr_canonicalize(&msg.warchest_addr)?
    })?;

    let pool_info: &PoolInfoRaw = &PoolInfoRaw {
        contract_addr: env.contract.address.clone(),
        liquidity_token: CanonicalAddr::from(vec![]),
        slippage: msg.slippage,
        asset_infos: [
            msg.asset_info.to_raw(deps.api)?,
            AssetInfo::NativeToken{ denom: LUNA_DENOM.to_string()}.to_raw(deps.api)?,
            AssetInfo::Token{ contract_addr: msg.aust_address }.to_raw(deps.api)?
        ],
    };
    POOL_INFO.save(deps.storage, pool_info)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: "test liquidity token".to_string(),
                symbol: "tLP".to_string(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })?,
            funds: vec![],
            label: "".to_string(),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> VaultResult {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::AbovePeg { amount, uaust_withdraw_amount } => try_arb_above_peg(deps, env, info, amount, uaust_withdraw_amount),
        ExecuteMsg::BelowPeg { amount, uaust_withdraw_amount } => try_arb_below_peg(deps, env, info, amount, uaust_withdraw_amount),
        ExecuteMsg::ProvideLiquidity{ asset } => try_provide_liquidity(deps, info, asset),
        ExecuteMsg::AnchorDeposit{ amount } => try_deposit_to_anchor(deps, info, amount),
        ExecuteMsg::SetSlippage{ slippage } => set_slippage(deps, info, slippage),
        ExecuteMsg::SetBurnAddress{ burn_addr } => set_burn_addr(deps, info, burn_addr),
        ExecuteMsg::SetAdmin{ admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
            Ok(Response::default().add_attribute("previous admin", previous_admin).add_attribute("admin", admin))
        },
    }
}


pub fn try_withdraw_liquidity(
    deps: DepsMut,
    _env: Env,
    sender: String,
    amount: Uint128,
) -> VaultResult {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;

    let lp_addr = deps.api.addr_humanize(&info.liquidity_token)?;
    let total_share: Uint128 = query_supply(&deps.querier, lp_addr.clone())?;
    let total_value: Uint128 = compute_total_value(deps.as_ref(), &info)?;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);
    let refund_amount: Uint128 = total_value * share_ratio;
    let fee_config = FEE.load(deps.storage)?;
    let withdraw_fee: Uint128 = fee_config.burn_fee.compute(refund_amount);
    let withdraw_fee_asset = Asset {
        info: AssetInfo::NativeToken{ denom: get_stable_denom(deps.as_ref())? },
        amount: withdraw_fee
    };
    let withdraw_fee_tax: Uint128 = withdraw_fee_asset.compute_tax(&deps.querier)?;

    let total_user_share = amount + query_token_balance(&deps.querier, lp_addr, deps.api.addr_validate(&sender)?)?;
    let total_user_share_ratio: Decimal = Decimal::from_ratio(total_user_share, total_share);
    let user_share: Uint128 = total_value * total_user_share_ratio;
    let raw_sender = deps.api.addr_canonicalize(sender.as_str())?;
    let key = &raw_sender.as_slice();
    let user_deposit = DEPOSIT_MANAGER.get(deps.storage, key)?;

    let user_profit = user_share - user_deposit;
    let user_profit_share_ratio = Decimal::from_ratio(amount, total_user_share);
    let user_profit_share = user_profit * user_profit_share_ratio;

    let white_whale_adjusted_refund_amount = refund_amount - withdraw_fee - withdraw_fee_tax;
    let withdraw_asset = Asset {
        info: AssetInfo::NativeToken{ denom: get_stable_denom(deps.as_ref())? },
        amount: white_whale_adjusted_refund_amount
    };
    DEPOSIT_MANAGER.decrease(deps.storage, key, refund_amount - user_profit_share)?;
    let refund_coin = withdraw_asset.deduct_tax(&deps.querier)?;

    let response = Response::new()
        .add_attribute("action", "withdraw_liquidity")
        .add_attribute("withdrawn_amount", refund_amount.to_string())
        .add_attribute("refund_amount", refund_coin.amount.to_string())
        .add_attribute("withdrawn_share", white_whale_adjusted_refund_amount.to_string())
        .add_attribute("withdrawn_deposit", (refund_amount - user_profit_share).to_string())
        .add_attribute("withdrawn_profit", user_profit_share.to_string())
        .add_attribute("tax", (white_whale_adjusted_refund_amount - refund_coin.amount).to_string())
        .add_attribute("withdraw fee tax", withdraw_fee_tax.to_string())
        .add_attribute("total_fee", withdraw_fee.to_string())
        .add_attribute("burn_vault_fee", withdraw_fee.to_string());
    // withdraw from anchor if necessary
    // TODO: Improve
    // let state = STATE.load(deps.storage)?;
    // let stable_balance: Uint128 = query_balance(&deps.querier, env.contract.address.clone(), get_stable_denom(deps.as_ref())?)?;
    // if refund_asset.amount*Decimal::from_ratio(Uint128::from(50u64), Uint128::from(1u64)) > stable_balance {
    //     let uaust_amount: Uint128 = query_token_balance(&deps.querier, deps.api.addr_humanize(&state.aust_address)?, env.contract.address)?;
    //     let uaust_exchange_rate_response = query_aust_exchange_rate(deps.as_ref(), deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string())?;
    //     let uaust_ust_rate = Decimal::from_str(&uaust_exchange_rate_response.exchange_rate.to_string())?;
    //     let uaust_amount_in_uust = uaust_amount * uaust_ust_rate;
    //     // TODO: Improve
    //     if uaust_amount_in_uust > Uint128::from(10u64 * 1000000u64) || amount == total_share {
    //         response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
    //             contract_addr: state.aust_address.to_string(),
    //             msg: to_binary(
    //                 &Cw20ExecuteMsg::Send{
    //                     contract: state.anchor_money_market_address.to_string(),
    //                     amount: uaust_amount,
    //                     msg: to_binary(&AnchorMsg::RedeemStable{})?
    //                 }
    //             )?,
    //             funds: vec![]
    //         }));
    //     }
    // }

    let refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: sender,
        amount: vec![refund_coin],
    });
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        funds: vec![],
    });
    let burn_vault_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&fee_config.burn_addr)?.to_string(),
        funds: vec![Coin{ denom: DEPOSIT_INFO.load(deps.storage)?.get_denom()?, amount: withdraw_fee }],
        msg: to_binary(&BurnMsg::Deposit{})?
    });
    // let warchest_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: deps.api.addr_humanize(&fee_config.warchest_addr)?.to_string(),
    //     funds: vec![Coin{ denom: DEPOSIT_INFO.load(deps.storage)?.get_denom()?, amount: profit_share }],
    //     msg: to_binary(&BurnMsg::Deposit{})?
    // });

    Ok(response.add_message(refund_msg).add_message(burn_msg).add_message(burn_vault_msg))
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> VaultResult {
        match from_binary(&cw20_msg.msg)? {
            Cw20HookMsg::Swap {
                ..
            } => {
                Err(StableVaultError::NoSwapAvailabe{})
            }
            Cw20HookMsg::WithdrawLiquidity {} => {
                let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
                if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != info.liquidity_token {
                    return Err(StableVaultError::Unauthorized{});
                }

                try_withdraw_liquidity(deps, env, cw20_msg.sender, cw20_msg.amount)
            }
        }
}

pub fn get_stable_denom(
    deps: Deps,
) -> StdResult<String> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let stable_info = info.asset_infos[0].to_normal(deps.api)?;
    let stable_denom = match stable_info {
        AssetInfo::Token{..} => String::default(),
        AssetInfo::NativeToken{denom} => denom
    };
    if stable_denom == String::default() {
        return Err(StdError::generic_err("get_stable_denom failed: No native token found."));
    }

    Ok(stable_denom)
}

pub fn get_slippage_ratio(slippage: Decimal) -> StdResult<Decimal> {
    Ok(Decimal::from_ratio(Uint128::from(100u64) - Uint128::from(100u64) * slippage, Uint128::from(100u64)))
}

pub fn add_profit_check(
    deps: Deps,
    response: Response<TerraMsgWrapper>,
    first_msg: CosmosMsg<TerraMsgWrapper>,
    second_msg: CosmosMsg<TerraMsgWrapper>
) -> VaultResult {
    let state = STATE.load(deps.storage)?;

    Ok(response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: deps.api.addr_humanize(&state.profit_check_address)?.to_string(),
        msg: to_binary(
            &ProfitCheckMsg::BeforeTrade{}
        )?,
        funds: vec![]
    }))
    .add_message(first_msg)
    .add_message(second_msg)
    .add_message(CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: deps.api.addr_humanize(&state.profit_check_address)?.to_string(),
        msg: to_binary(
            &ProfitCheckMsg::AfterTrade{}
        )?,
        funds: vec![]
    })))
}

pub fn try_arb_below_peg(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    amount: Coin,
    uaust_withdraw_amount: Uint128
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.trader {
        return Err(StableVaultError::Unauthorized{});
    }

    let ask_denom = LUNA_DENOM.to_string();

    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let slippage = info.slippage;
    let slippage_ratio = get_slippage_ratio(slippage)?;
    let expected_luna_amount = query_market_price(deps.as_ref(), amount.clone(), LUNA_DENOM.to_string())? * slippage_ratio;
    let luna_pool_price = simulate_terraswap_swap(deps.as_ref(), deps.api.addr_humanize(&state.pool_address)?, Coin{denom: LUNA_DENOM.to_string(), amount: expected_luna_amount})?;

    let swap_msg = create_swap_msg(
        amount,
        ask_denom.clone(),
    );
    let residual_luna = query_balance(&deps.querier, env.contract.address, LUNA_DENOM.to_string())?;
    let offer_coin = Coin{ denom: ask_denom, amount: residual_luna + expected_luna_amount};
    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.pool_address)?.to_string(),
        funds: vec![offer_coin.clone()],
        msg: to_binary(&create_terraswap_msg(offer_coin, Decimal::from_ratio(luna_pool_price, expected_luna_amount), Some(slippage)))?,
    });

    let mut response = Response::new();
    if uaust_withdraw_amount > Uint128::zero() {
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
            contract_addr: deps.api.addr_humanize(&state.aust_address)?.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::Send{
                    contract: deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string(),
                    amount: uaust_withdraw_amount,
                    msg: to_binary(&AnchorMsg::RedeemStable{})?
                }
            )?,
            funds: vec![]
        }));
    }
    add_profit_check(deps.as_ref(), response, swap_msg, terraswap_msg)
}

pub fn try_arb_above_peg(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    amount: Coin,
    uaust_withdraw_amount: Uint128
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.trader {
        return Err(StableVaultError::Unauthorized{});
    }

    let ask_denom = LUNA_DENOM.to_string();

    let expected_luna_amount = simulate_terraswap_swap(deps.as_ref(), deps.api.addr_humanize(&state.pool_address)?, amount.clone())?;
    let luna_pool_price = Decimal::from_ratio(amount.amount, expected_luna_amount);
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let slippage = info.slippage;

    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.pool_address)?.to_string(),
        funds: vec![amount.clone()],
        msg: to_binary(&create_terraswap_msg(amount.clone(), luna_pool_price, Some(slippage)))?,
    });

    let residual_luna = query_balance(&deps.querier, env.contract.address, LUNA_DENOM.to_string())?;
    let slippage_ratio = get_slippage_ratio(slippage)?;
    let offer_coin = Coin{ denom: ask_denom, amount: residual_luna + expected_luna_amount * slippage_ratio};

    let swap_msg = create_swap_msg(
        offer_coin,
        amount.denom,
    );

    let mut response = Response::new();
    if uaust_withdraw_amount > Uint128::zero() {
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
            contract_addr: deps.api.addr_humanize(&state.aust_address)?.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::Send{
                    contract: deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string(),
                    amount: uaust_withdraw_amount,
                    msg: to_binary(&AnchorMsg::RedeemStable{})?
                }
            )?,
            funds: vec![]
        }));
    }
    add_profit_check(deps.as_ref(), response, terraswap_msg, swap_msg)
}


/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    if msg.id == INSTANTIATE_REPLY_ID {
        let data = msg.result.unwrap().data.unwrap();
        let res: MsgInstantiateContractResponse =
            Message::parse_from_bytes(data.as_slice()).map_err(|_| {
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

// const COMMISSION_RATE: &str = "0.003";
// fn compute_swap(
//     offer_pool: Uint128,
//     ask_pool: Uint128,
//     offer_amount: Uint128,
// ) -> StdResult<(Uint128, Uint128, Uint128)> {
//     // offer => ask
//     // ask_amount = (ask_pool - cp / (offer_pool + offer_amount)) * (1 - commission_rate)
//     let cp = Uint128(offer_pool.u128() * ask_pool.u128());
//     let return_amount = (ask_pool - cp.multiply_ratio(1u128, offer_pool + offer_amount))?;

//     // calculate spread & commission
//     let spread_amount: Uint128 = (offer_amount * Decimal::from_ratio(ask_pool, offer_pool)
//         - return_amount)
//         .unwrap_or_else(|_| Uint128::zero());
//     let commission_amount: Uint128 = return_amount * Decimal::from_str(&COMMISSION_RATE).unwrap();

//     // commission will be absorbed to pool
//     let return_amount: Uint128 = (return_amount - commission_amount).unwrap();

//     Ok((return_amount, spread_amount, commission_amount))
// }

pub fn compute_total_value(
    deps: Deps,
    info: &PoolInfoRaw
) -> StdResult<Uint128> {
    let stable_info = info.asset_infos[0].to_normal(deps.api)?;
    let stable_denom = match stable_info {
        AssetInfo::Token{..} => String::default(),
        AssetInfo::NativeToken{denom} => denom
    };
    let stable_amount = query_balance(&deps.querier, info.contract_addr.clone(), stable_denom.clone())?;

    let luna_info = info.asset_infos[1].to_normal(deps.api)?;
    let luna_amount = match luna_info {
        AssetInfo::Token{..} => Uint128::zero(),
        AssetInfo::NativeToken{denom} => query_balance(&deps.querier, info.contract_addr.clone(), denom)?,
    };
    let luna_price = from_micro(query_market_price(deps, Coin{ denom: LUNA_DENOM.to_string(), amount: Uint128::from(1000000u64)}, stable_denom)?);
    let luna_value_in_stable = luna_amount * luna_price;

    let aust_info = info.asset_infos[2].to_normal(deps.api)?;
    let aust_amount = match aust_info {
        AssetInfo::Token{contract_addr} => query_token_balance(&deps.querier, deps.api.addr_validate(&contract_addr)?, info.contract_addr.clone())?,
        AssetInfo::NativeToken{..} => Uint128::zero()
    };

    let state = STATE.load(deps.storage)?;
    let epoch_state_response = query_aust_exchange_rate(deps, deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string())?;
    let aust_exchange_rate = Decimal::from_str(&epoch_state_response.exchange_rate.to_string())?;
    let aust_value_in_ust = aust_exchange_rate*aust_amount;

    let total_deposits_in_ust = stable_amount + luna_value_in_stable + aust_value_in_ust;
    Ok(total_deposits_in_ust)
}

pub fn try_provide_liquidity(
    deps: DepsMut,
    msg_info: MessageInfo,
    asset: Asset
) -> VaultResult {
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    deposit_info.assert(&asset.info)?;
    asset.assert_sent_native_token_balance(&msg_info)?;

    let fee_config = FEE.load(deps.storage)?;
    let deposit_fee = fee_config.burn_fee.compute(asset.amount);
    let deposit_asset: Asset = Asset{
        info: AssetInfo::NativeToken{ denom: get_stable_denom(deps.as_ref())? },
        amount: deposit_fee
    };
    let deposit_fee_tax: Uint128 = deposit_asset.compute_tax(&deps.querier)?;

    let deposit: Uint128 = asset.amount - deposit_fee - deposit_fee_tax;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let total_deposits_in_ust: Uint128 = compute_total_value(deps.as_ref(), &info)? - deposit_fee - deposit_fee_tax;

    let raw_sender = deps.api.addr_canonicalize(msg_info.sender.as_str())?;
    let key = &raw_sender.as_slice();
    DEPOSIT_MANAGER.increase(deps.storage, key, deposit)?;

    let total_share = query_supply(&deps.querier, deps.api.addr_humanize(&info.liquidity_token)?)?;
    let share = if total_share == Uint128::zero() {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_ust - deposit)
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
    // send fees to burn vault
    let fee_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&fee_config.burn_addr)?.to_string(),
        funds: vec![Coin{ denom: deposit_info.get_denom()?, amount: deposit_fee }],
        msg: to_binary(&BurnMsg::Deposit{})?
    });
    Ok(Response::new().add_attribute("deposit", deposit.to_string()).add_attribute("total_deposits", total_deposits_in_ust.to_string()).add_attribute("fee", deposit_fee.to_string()).add_attribute("tax", deposit_fee_tax.to_string()).add_message(msg).add_message(fee_msg))
}

pub fn try_deposit_to_anchor(
    deps: DepsMut,
    msg_info: MessageInfo,
    amount: Coin
) -> VaultResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let state = STATE.load(deps.storage)?;
    Ok(try_deposit(deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string(), amount)?)
}

pub fn set_slippage(
    deps: DepsMut,
    msg_info: MessageInfo,
    slippage: Decimal
) -> VaultResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    info.slippage = slippage;
    POOL_INFO.save(deps.storage, &info)?;
    Ok(Response::default())
}

pub fn set_burn_addr(
    deps: DepsMut,
    msg_info: MessageInfo,
    burn_addr: String
) -> VaultResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut fee_config = FEE.load(deps.storage)?;
    fee_config.burn_addr = deps.api.addr_canonicalize(&burn_addr)?;
    FEE.save(deps.storage, &fee_config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config{} => to_binary(&try_query_config(deps)?),
        QueryMsg::Pool{} => to_binary(&try_query_pool(deps)?),
        QueryMsg::Deposit{ addr } => to_binary(&try_query_deposit(deps, addr)?),
    }
}

pub fn try_query_config(
    deps: Deps
) -> StdResult<PoolInfo> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    info.to_normal(deps)
}

pub fn try_query_pool(
    deps: Deps
) -> StdResult<PoolResponse> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let assets: [Asset; 3] = info.query_pools(deps, info.contract_addr.clone())?;
    let total_share: Uint128 =
        query_supply(&deps.querier, deps.api.addr_humanize(&info.liquidity_token)?)?;

    let total_deposits_in_ust = compute_total_value(deps, &info)?;

    Ok(PoolResponse { assets, total_deposits_in_ust, total_share })
}

pub fn try_query_deposit(
    deps: Deps,
    addr: String
) -> StdResult<DepositResponse> {
    let raw_sender = deps.api.addr_canonicalize(addr.as_str())?;
    let key = &raw_sender.as_slice();
    let deposit = DEPOSIT_MANAGER.get(deps.storage, key)?;
    Ok(DepositResponse{deposit})
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_env};
    use crate::testing::{mock_dependencies};
    use cosmwasm_std::{Uint128, Api};
    use terra_cosmwasm::TerraRoute;
    use terraswap::asset::AssetInfo;

    fn get_test_init_msg() -> InitMsg {
        InitMsg {
            pool_address: "test_pool".to_string(),
            anchor_money_market_address: "test_mm".to_string(),
            aust_address: "test_aust".to_string(),
            seignorage_address: "test_seignorage".to_string(),
            profit_check_address: "test_profit_check".to_string(),
            burn_addr: "burn".to_string(),
            warchest_addr: "warchest".to_string(),
            asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() },
            slippage: Decimal::percent(1u64), token_code_id: 0u64,
            denom: "uusd".to_string(),
            warchest_fee: Decimal::percent(10u64),
            burn_vault_fee: Decimal::permille(5u64),
            max_burn_vault_fee: Uint128::from(1000000u64)
        }
    }

    #[test]
    fn test_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn test_set_slippage() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let msg_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());

        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::percent(1u64));

        let msg = ExecuteMsg::SetSlippage {
            slippage: Decimal::one()
        };
        let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::one());
    }

    #[test]
    fn when_given_a_below_peg_msg_then_handle_returns_first_a_mint_then_a_terraswap_msg() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let msg_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let _res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();

        let msg = ExecuteMsg::BelowPeg {
            amount: Coin{denom: "uusd".to_string(), amount: Uint128::from(1000000u64)},
            uaust_withdraw_amount: Uint128::zero()
        };

        let res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
        assert_eq!(4, res.messages.len());
        let second_msg = res.messages[1].msg.clone();
        match second_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
            CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected"),
            _ => panic!("unexpected"),
        }
        let second_msg = res.messages[2].msg.clone();
        match second_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(_t) => panic!("unexpected"),
            CosmosMsg::Wasm(_wasm_msg) => {},
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn when_given_an_above_peg_msg_then_handle_returns_first_a_terraswap_then_a_mint_msg() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let msg_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let _res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();

        let msg = ExecuteMsg::AbovePeg {
            amount: Coin{denom: "uusd".to_string(), amount: Uint128::from(1000000u64)},
            uaust_withdraw_amount: Uint128::zero()
        };

        let res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
        assert_eq!(4, res.messages.len());
        let second_msg = res.messages[1].msg.clone();
        match second_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(_t) => panic!("unexpected"),
            CosmosMsg::Wasm(_wasm_msg) => {},
            _ => panic!("unexpected"),
        }
        let third_msg = res.messages[2].msg.clone();
        match third_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
            CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected"),
            _ => panic!("unexpected"),
        }
    }
}
