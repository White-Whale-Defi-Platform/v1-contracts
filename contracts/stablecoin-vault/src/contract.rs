use cosmwasm_std::{ entry_point, CanonicalAddr,
    from_binary, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Fraction, MessageInfo, Response, StdError,
    StdResult, WasmMsg, Uint128, Decimal, SubMsg, Reply, ReplyOn
};
use terra_cosmwasm::{create_swap_msg, TerraMsgWrapper};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg;
use terraswap::querier::{query_balance, query_token_balance, query_supply};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};

use white_whale::community_fund::msg::ExecuteMsg as CommunityFundMsg;
use white_whale::denom::LUNA_DENOM;
use white_whale::deposit_info::DepositInfo;
use white_whale::fee::{Fee, CappedFee, VaultFee};
use white_whale::msg::{create_terraswap_msg, VaultQueryMsg as QueryMsg, AnchorMsg, EstimateDepositFeeResponse, EstimateWithdrawFeeResponse, FeeResponse};
use white_whale::query::terraswap::simulate_swap as simulate_terraswap_swap;
use white_whale::query::terraswap::pool_ratio;
use white_whale::query::anchor::query_aust_exchange_rate;
use white_whale::profit_check::msg::{HandleMsg as ProfitCheckMsg};
use white_whale::anchor::try_deposit_to_anchor as try_deposit;

use crate::error::StableVaultError;
use crate::msg::{ExecuteMsg, InitMsg, PoolResponse};
use crate::state::{State, ADMIN, STATE, POOL_INFO, DEPOSIT_INFO, FEE};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::querier::{query_market_price};
use crate::response::MsgInstantiateContractResponse;

use std::cmp::min;


const INSTANTIATE_REPLY_ID: u64 = 1;
const DEFAULT_LP_TOKEN_NAME: &str = "White Whale UST Vault LP Token";
const DEFAULT_LP_TOKEN_SYMBOL: &str = "wwVUst";

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
        anchor_min_withdraw_amount: msg.anchor_min_withdraw_amount,
    };
    // Store the initial config
    STATE.save(deps.storage, &state)?;
    DEPOSIT_INFO.save(deps.storage, &DepositInfo{
        asset_info: msg.asset_info.clone()
    })?;
    // Setup the fees system with a fee and other contract addresses
    FEE.save(deps.storage, &VaultFee{
        community_fund_fee: CappedFee{
            fee: Fee{ share: msg.community_fund_fee },
            max_fee: msg.max_community_fund_fee
        },
        warchest_fee: Fee{ share: msg.warchest_fee },
        community_fund_addr: deps.api.addr_canonicalize(&msg.community_fund_addr)?,
        warchest_addr: deps.api.addr_canonicalize(&msg.warchest_addr)?
    })?;

    // Setup and save the relevant pools info in state. The saved pool will be the one used by the vault.
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
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    // Both the lp_token_name and symbol are Options, attempt to unwrap their value falling back to the default if not provided
    let lp_token_name: String = msg.vault_lp_token_name.unwrap_or(String::from(DEFAULT_LP_TOKEN_NAME));
    let lp_token_symbol: String = msg.vault_lp_token_symbol.unwrap_or(String::from(DEFAULT_LP_TOKEN_SYMBOL));

    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: lp_token_name.to_string(),
                symbol: lp_token_symbol.to_string(),
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
        ExecuteMsg::SetAdmin{ admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
            Ok(Response::default().add_attribute("previous admin", previous_admin).add_attribute("admin", admin))
        },
        ExecuteMsg::SetTrader{ trader } => set_trader(deps, info, trader),
        ExecuteMsg::SetFee{ community_fund_fee, warchest_fee } => set_fee(deps, info, community_fund_fee, warchest_fee),
    }
}

pub fn try_provide_liquidity(
    deps: DepsMut,
    msg_info: MessageInfo,
    asset: Asset
) -> VaultResult {
    let deposit_info = DEPOSIT_INFO.load(deps.storage)?;
    deposit_info.assert(&asset.info)?;
    asset.assert_sent_native_token_balance(&msg_info)?;

    let deposit_fee = compute_transaction_fee(deps.as_ref(), asset.amount)?;
    let deposit: Uint128 = asset.amount - deposit_fee;

    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let total_deposits_in_ust: Uint128 = compute_total_value(deps.as_ref(), &info)?;

    let total_share = query_supply(&deps.querier, deps.api.addr_humanize(&info.liquidity_token)?)?;
    let share = if total_share == Uint128::zero() {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_ust - asset.amount)
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
    // send fees to community fund
    let denom = deposit_info.get_denom()?;
    let fee_config = FEE.load(deps.storage)?;
    let community_fund_asset = Asset{
        info: AssetInfo::NativeToken{ denom },
        amount: deposit_fee
    };
    let community_fund_fee_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&fee_config.community_fund_addr)?.to_string(),
        funds: vec![community_fund_asset.deduct_tax(&deps.querier)?],
        msg: to_binary(&CommunityFundMsg::Deposit{})?
    });
    Ok(
        Response::new().add_attribute("deposit", deposit.to_string()).add_attribute("total_deposits", total_deposits_in_ust.to_string()).add_attribute("fee", deposit_fee.to_string()).add_message(msg)
        .add_message(community_fund_fee_msg)
    )
}

/// attempt to withdraw deposits. Fees are calculated and deducted and the net refund is sent 
/// a withdrawal from Anchor Money Market may be performed as a part of the withdrawal process.
fn try_withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    sender: String,
    amount: Uint128,
) -> VaultResult {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;

    let lp_addr = deps.api.addr_humanize(&info.liquidity_token)?;
    let total_share: Uint128 = query_supply(&deps.querier, lp_addr)?;
    let total_value: Uint128 = compute_total_value(deps.as_ref(), &info)?;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);
    let refund_amount: Uint128 = total_value * share_ratio;
    let community_fund_fee = compute_transaction_fee(deps.as_ref(), refund_amount)?;
    let warchest_fee = compute_warchest_fee(deps.as_ref(), refund_amount)?;
    let net_refund_amount = refund_amount - community_fund_fee - warchest_fee;

    let mut response = Response::new();
    // // withdraw from anchor if necessary
    // // TODO: Improve
    let state = STATE.load(deps.storage)?;
    let denom = DEPOSIT_INFO.load(deps.storage)?.get_denom()?;
    let uaust_amount: Uint128 = query_token_balance(&deps.querier, deps.api.addr_humanize(&state.aust_address)?, env.contract.address.clone())?;
    if uaust_amount > Uint128::zero() {
        let stable_balance: Uint128 = query_balance(&deps.querier, env.contract.address.clone(), denom.clone())?;
        let stable_ratio = Decimal::from_ratio(stable_balance, total_value);
        let anchor_ratio = Decimal::one() - stable_ratio;
        let anchor_withdraw_amount = refund_amount * anchor_ratio;
        if anchor_withdraw_amount > state.anchor_min_withdraw_amount {
            let uaust_exchange_rate_response = query_aust_exchange_rate(deps.as_ref(), deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string())?;
            let ust_uaust_rate = Decimal::from(uaust_exchange_rate_response.exchange_rate).inv().unwrap();
            let max_aust_amount = query_token_balance(&deps.querier, deps.api.addr_humanize(&state.aust_address)?, env.contract.address)?;
            let anchor_withdraw_aust_amount = min(anchor_withdraw_amount * ust_uaust_rate, max_aust_amount);
            response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
                contract_addr: deps.api.addr_humanize(&state.aust_address)?.to_string(),
                msg: to_binary(
                    &Cw20ExecuteMsg::Send{
                        contract: deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string(),
                        amount: anchor_withdraw_aust_amount,
                        msg: to_binary(&AnchorMsg::RedeemStable{})?
                    }
                )?,
                funds: vec![]
            })).add_attribute("anchor withdrawal", anchor_withdraw_aust_amount.to_string()).add_attribute("ust_aust_rate", ust_uaust_rate.to_string())
        }
    }

    let refund_asset = Asset{
        info: AssetInfo::NativeToken{ denom: denom.clone() },
        amount: net_refund_amount
    };

    // Prepare refund message 
    let refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: sender,
        amount: vec![refund_asset.deduct_tax(&deps.querier)?],
    });

    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        funds: vec![],
    });

    let community_fund_asset = Asset{
        info: AssetInfo::NativeToken{ denom: denom.clone() },
        amount: community_fund_fee
    };

    let warchest_asset = Asset{
        info: AssetInfo::NativeToken{ denom },
        amount: warchest_fee
    };

    // Prepare deposit messages for warchest and community fund.
    let fee_config = FEE.load(deps.storage)?;
    let community_fund_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&fee_config.community_fund_addr)?.to_string(),
        funds: vec![community_fund_asset.deduct_tax(&deps.querier)?],
        msg: to_binary(&CommunityFundMsg::Deposit{})?
    });
    let warchest_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&fee_config.warchest_addr)?.to_string(),
        funds: vec![warchest_asset.deduct_tax(&deps.querier)?],
        msg: to_binary(&CommunityFundMsg::Deposit{})?
    });

    Ok(response.add_message(refund_msg).add_message(burn_msg).add_message(community_fund_msg).add_message(warchest_msg)
        .add_attribute("action", "withdraw_liquidity")
        .add_attribute("withdrawn_amount", refund_amount.to_string())
        .add_attribute("refund amount", net_refund_amount.to_string())
        .add_attribute("community fund fee", community_fund_fee.to_string())
        .add_attribute("warchest fee", warchest_fee.to_string())
    )
}

/// handler function invoked when the stablecoin-vault contract receives
/// a transaction. This is akin to a payable function in Solidity
fn receive_cw20(
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

fn get_slippage_ratio(slippage: Decimal) -> StdResult<Decimal> {
    Ok(Decimal::from_ratio(Uint128::from(100u64) - Uint128::from(100u64) * slippage, Uint128::from(100u64)))
}

/// helper method which takes two msgs assumed to be Terraswap trades
/// and then composes a response with a ProfitCheck BeforeTrade and AfterTrade
/// the result is an OK'd response with a series of msgs in this order
/// Profit Check before trade - first_msg - second_msg - Profit Check after trade
fn add_profit_check(
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

// attempt to perform an arbitrage operation with the assumption that 
// the currency to be arb'd is below peg. This is important as checks are 
// performed to ensure the arb opportunity still exists and price is indeed below peg
// if needed, funds are withdrawn from anchor and messages are prepared to perform the swaps 
// Before sending; the profit check contract messages are also added 
// by providing the swap msg and terraswap msg to add_profit_check func
fn try_arb_below_peg(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    amount: Coin,
    uaust_withdraw_amount: Uint128
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    // Ensure the caller is a named Trader
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.trader {
        return Err(StableVaultError::Unauthorized{});
    }

    let ask_denom = LUNA_DENOM.to_string();

    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    // Store slippage 
    let slippage = info.slippage;
    let slippage_ratio = get_slippage_ratio(slippage)?;
    // Check how much we can Luna we can get accounting for slippage
    let expected_luna_amount = query_market_price(deps.as_ref(), amount.clone(), LUNA_DENOM.to_string())? * slippage_ratio;
    let luna_pool_price = simulate_terraswap_swap(deps.as_ref(), deps.api.addr_humanize(&state.pool_address)?, Coin{denom: LUNA_DENOM.to_string(), amount: expected_luna_amount})?;

    let swap_msg = create_swap_msg(
        amount,
        ask_denom.clone(),
    );
    let residual_luna = query_balance(&deps.querier, env.contract.address, LUNA_DENOM.to_string())?;
    let offer_coin = Coin{ denom: ask_denom, amount: residual_luna + expected_luna_amount};
    
    // Prepare a terraswap message to swap an offer_coin for luna
    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.pool_address)?.to_string(),
        funds: vec![offer_coin.clone()],
        msg: to_binary(&create_terraswap_msg(offer_coin, Decimal::from_ratio(luna_pool_price, expected_luna_amount), Some(slippage)))?,
    });

    let mut response = Response::new();
    if uaust_withdraw_amount > Uint128::zero() {
        // Attempt to remove some money from anchor 
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
    // Finish off all the above by wrapping the swap and terraswap messages in between the 2 profit check queries
    add_profit_check(deps.as_ref(), response, swap_msg, terraswap_msg)
}

// attempt to perform an arbitrage operation with the assumption that 
// the currency to be arbed is above peg. This is important as checks are 
// performed to ensure the arb opportunity still exists and price is indeed above peg
// if needed, funds are withdrawn from anchor and messages are prepared to perform the swaps 
// Before sending; the profit check contract messages are also added 
// by providing the swapmsg and terraswap msg to add_profit_check func
fn try_arb_above_peg(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    amount: Coin,
    uaust_withdraw_amount: Uint128
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    // Ensure the caller is a named Trader
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.trader {
        return Err(StableVaultError::Unauthorized{});
    }


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

    let ask_denom = LUNA_DENOM.to_string();
    let offer_coin = Coin{ denom: ask_denom, amount: residual_luna + expected_luna_amount * slippage_ratio};

    // Prepare a swap message to swap an offer_coin for luna
    let swap_msg = create_swap_msg(
        offer_coin,
        amount.denom,
    );

    let mut response = Response::new();
    if uaust_withdraw_amount > Uint128::zero() {
        // Attempt to remove some money from anchor 
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
    // Finish off all the above by wrapping the swap and terraswap messages in between the 2 profit check queries
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

// compute total value of deposits in UST and return
pub fn compute_total_value(
    deps: Deps,
    info: &PoolInfoRaw
) -> StdResult<Uint128> {
    let state = STATE.load(deps.storage)?;
    let stable_info = info.asset_infos[0].to_normal(deps.api)?;
    let stable_denom = match stable_info {
        AssetInfo::Token{..} => String::default(),
        AssetInfo::NativeToken{denom} => denom
    };
    let stable_amount = query_balance(&deps.querier, info.contract_addr.clone(), stable_denom)?;

    let luna_info = info.asset_infos[1].to_normal(deps.api)?;
    let luna_amount = match luna_info {
        AssetInfo::Token{..} => Uint128::zero(),
        AssetInfo::NativeToken{denom} => query_balance(&deps.querier, info.contract_addr.clone(), denom)?,
    };
    //let luna_price = from_micro(query_market_price(deps, Coin{ denom: LUNA_DENOM.to_string(), amount: Uint128::from(1000000u64)}, stable_denom)?);
    // Get on-chain luna/ust price
    let luna_price = pool_ratio(deps, deps.api.addr_humanize(&state.pool_address)?)?;
    let luna_value_in_stable = luna_amount * luna_price;

    let aust_info = info.asset_infos[2].to_normal(deps.api)?;
    let aust_amount = match aust_info {
        AssetInfo::Token{contract_addr} => query_token_balance(&deps.querier, deps.api.addr_validate(&contract_addr)?, info.contract_addr.clone())?,
        AssetInfo::NativeToken{..} => Uint128::zero()
    };

    
    let epoch_state_response = query_aust_exchange_rate(deps, deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string())?;
    let aust_exchange_rate = Decimal::from(epoch_state_response.exchange_rate);
    let aust_value_in_ust = aust_exchange_rate*aust_amount;

    let total_deposits_in_ust = stable_amount + luna_value_in_stable + aust_value_in_ust;
    Ok(total_deposits_in_ust)
}

// compute the community fund fee
pub fn compute_transaction_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let fee_config = FEE.load(deps.storage)?;
    let fee = fee_config.community_fund_fee.compute(amount);
    Ok(fee)
}

// compute the war chest fee
pub fn compute_warchest_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let fee_config = FEE.load(deps.storage)?;
    let fee = fee_config.warchest_fee.compute(amount);
    Ok(fee)
}

// compute the withdrawal fee
pub fn compute_withdraw_fee(deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    let community_fund_fee = compute_transaction_fee(deps, amount)?;
    let warchest_fee = compute_warchest_fee(deps, amount)?;
    Ok(community_fund_fee + warchest_fee)
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

/// Setters for contract parameters/config values

pub fn set_slippage(
    deps: DepsMut,
    msg_info: MessageInfo,
    slippage: Decimal
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let previous_slippage = info.slippage;
    info.slippage = slippage;
    POOL_INFO.save(deps.storage, &info)?;
    Ok(Response::new().add_attribute("slippage", slippage.to_string()).add_attribute("previous slippage", previous_slippage.to_string()))
}

pub fn set_trader(
    deps: DepsMut,
    msg_info: MessageInfo,
    trader: String
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    // Get the old trader 
    let previous_trader = deps.api.addr_humanize(&state.trader)?.to_string();
    // Store the new trader, validating it is indeed an address along the way
    state.trader = deps.api.addr_canonicalize(&trader)?;
    STATE.save(deps.storage, &state)?;
    // Respond and note the previous traders address
    Ok(Response::new().add_attribute("trader", trader).add_attribute("previous trader", previous_trader))
}

pub fn set_fee(
    deps: DepsMut,
    msg_info: MessageInfo,
    community_fund_fee: Option<CappedFee>,
    warchest_fee: Option<Fee>,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    // TODO: Evaluate this.
    let mut fee_config = FEE.load(deps.storage)?;
    if let Some(fee) = community_fund_fee {
        fee_config.community_fund_fee = fee;
    }
    if let Some(fee) = warchest_fee {
        fee_config.warchest_fee = fee;
    }
    FEE.save(deps.storage, &fee_config)?;
    Ok(Response::default())
}

/// Query Handler and query functions 

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config{} => to_binary(&try_query_config(deps)?),
        QueryMsg::Pool{} => to_binary(&try_query_pool(deps)?),
        QueryMsg::Fees{} => to_binary(&query_fees(deps)?),
        QueryMsg::EstimateDepositFee{ amount } => to_binary(&estimate_deposit_fee(deps, amount)?),
        QueryMsg::EstimateWithdrawFee{ amount } => to_binary(&estimate_withdraw_fee(deps, amount)?),
    }
}

pub fn query_fees(deps: Deps) -> StdResult<FeeResponse> {
    Ok(FeeResponse{
        fees: FEE.load(deps.storage)?
    })
}

pub fn estimate_deposit_fee(deps: Deps, amount: Uint128) -> StdResult<EstimateDepositFeeResponse> {
    let fee = compute_transaction_fee(deps, amount)?;
    Ok(EstimateDepositFeeResponse{
        fee: vec![Coin{ denom: DEPOSIT_INFO.load(deps.storage)?.get_denom()?, amount: fee }]
    })
}

pub fn estimate_withdraw_fee(deps: Deps, amount: Uint128) -> StdResult<EstimateWithdrawFeeResponse> {
    let fee = compute_withdraw_fee(deps, amount)?;
    Ok(EstimateWithdrawFeeResponse{
        fee: vec![Coin{ denom: DEPOSIT_INFO.load(deps.storage)?.get_denom()?, amount: fee }]
    })
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

    let total_value_in_ust = compute_total_value(deps, &info)?;

    Ok(PoolResponse { assets, total_value_in_ust, total_share })
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
            community_fund_addr: "community_fund".to_string(),
            warchest_addr: "warchest".to_string(),
            asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() },
            slippage: Decimal::percent(1u64), token_code_id: 0u64,
            warchest_fee: Decimal::percent(10u64),
            community_fund_fee: Decimal::permille(5u64),
            max_community_fund_fee: Uint128::from(1000000u64),
            anchor_min_withdraw_amount: Uint128::from(10000000u64),
            vault_lp_token_name: None,
            vault_lp_token_symbol: None
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
    fn test_init_with_non_default_vault_lp_token() {
        let mut deps = mock_dependencies(&[]);

        let custom_token_name = String::from("My LP Token");
        let custom_token_symbol = String::from("MyLP");

        // Define a custom Init Msg with the custom token info provided
        let msg = InitMsg {
            pool_address: "test_pool".to_string(),
            anchor_money_market_address: "test_mm".to_string(),
            aust_address: "test_aust".to_string(),
            seignorage_address: "test_seignorage".to_string(),
            profit_check_address: "test_profit_check".to_string(),
            community_fund_addr: "community_fund".to_string(),
            warchest_addr: "warchest".to_string(),
            asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() },
            slippage: Decimal::percent(1u64), token_code_id: 0u64,
            warchest_fee: Decimal::percent(10u64),
            community_fund_fee: Decimal::permille(5u64),
            max_community_fund_fee: Uint128::from(1000000u64),
            anchor_min_withdraw_amount: Uint128::from(10000000u64),
            vault_lp_token_name: Some(custom_token_name.clone()),
            vault_lp_token_symbol: Some(custom_token_symbol.clone())
        };

        // Prepare mock env 
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        // Ensure we have 1 message
        assert_eq!(1, res.messages.len());
        // Verify the message is the one we expect but also that our custom provided token name and symbol were taken into account.
        assert_eq!(
            res.messages,
            vec![SubMsg {
                // Create LP token
                msg: WasmMsg::Instantiate {
                    admin: None,
                    code_id: msg.token_code_id,
                    msg: to_binary(&TokenInstantiateMsg {
                        name: custom_token_name.to_string(),
                        symbol: custom_token_symbol.to_string(),
                        decimals: 6,
                        initial_balances: vec![],
                        mint: Some(MinterResponse {
                            minter: env.contract.address.to_string(),
                            cap: None,
                        }),
                    })
                    .unwrap(),
                    funds: vec![],
                    label: "".to_string(),
                }
                .into(),
                gas_limit: None,
                id: INSTANTIATE_REPLY_ID,
                reply_on: ReplyOn::Success,
            }]
        );
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
    fn test_set_warchest_fee() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let msg_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());

        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::percent(1u64));

        let warchest_fee = FEE.load(&deps.storage).unwrap().warchest_fee.share;
        let new_fee =  Decimal::permille(1u64);
        assert_ne!(warchest_fee, new_fee);
        let msg = ExecuteMsg::SetFee {
            community_fund_fee: None,
            warchest_fee: Some(Fee { share: new_fee })
        };
        let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
        let warchest_fee = FEE.load(&deps.storage).unwrap().warchest_fee.share;
        assert_eq!(warchest_fee, new_fee);
    }

    #[test]
    fn test_set_community_fund_fee() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let msg_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());

        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::percent(1u64));

        let community_fund_fee = FEE.load(&deps.storage).unwrap().community_fund_fee.fee.share;
        let new_fee =  Decimal::permille(1u64);
        let new_max_fee = Uint128::from(42u64);
        assert_ne!(community_fund_fee, new_fee);
        let msg = ExecuteMsg::SetFee {
            community_fund_fee:  Some(CappedFee { fee: Fee{ share: new_fee }, max_fee: new_max_fee }),
            warchest_fee: None
        };
        let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
        let community_fund_fee = FEE.load(&deps.storage).unwrap().community_fund_fee.fee.share;
        let community_fund_max_fee = FEE.load(&deps.storage).unwrap().community_fund_fee.max_fee;
        assert_eq!(community_fund_fee, new_fee);
        assert_eq!(community_fund_max_fee, new_max_fee);
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
