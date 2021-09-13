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
use white_whale::denom::{ UST_DENOM, LUNA_DENOM };
use white_whale::msg::{create_terraswap_msg, VaultQueryMsg as QueryMsg, AnchorMsg};
use white_whale::query::terraswap::simulate_swap as simulate_terraswap_swap;
use white_whale::query::anchor::query_aust_exchange_rate;
use white_whale::profit_check::msg::{HandleMsg as ProfitCheckMsg, QueryMsg as ProfitCheckQueryMsg, LastProfitResponse};
use white_whale::anchor::try_deposit_to_anchor as try_deposit;

use crate::error::StableVaultError;
use crate::msg::{HandleMsg, InitMsg, PoolResponse};
use crate::state::{State, STATE, POOL_INFO};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::querier::{query_market_price, from_micro};
use crate::response::MsgInstantiateContractResponse;
use std::str::FromStr;


const INSTANTIATE_REPLY_ID: u64 = 1;
const TRADE_REPLY_ID: u64 = 2;

type VaultResult = Result<Response<TerraMsgWrapper>, StableVaultError>;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        trader: deps.api.addr_canonicalize(info.sender.as_str())?,
        pool_address: deps.api.addr_canonicalize(&msg.pool_address)?,
        anchor_money_market_address: deps.api.addr_canonicalize(&msg.anchor_money_market_address)?,
        aust_address: deps.api.addr_canonicalize(&msg.aust_address)?,
        seignorage_address: deps.api.addr_canonicalize(&msg.seignorage_address)?,
        profit_check_address: deps.api.addr_canonicalize(&msg.profit_check_address)?,
        burn_addr: deps.api.addr_canonicalize(&msg.burn_addr)?,
        profit_burn_ratio: msg.profit_burn_ratio
    };

    STATE.save(deps.storage, &state)?;

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
    msg: HandleMsg,
) -> VaultResult {
    match msg {
        HandleMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        HandleMsg::AbovePeg { amount, uaust_withdraw_amount } => try_arb_above_peg(deps, env, info, amount, uaust_withdraw_amount),
        HandleMsg::BelowPeg { amount, uaust_withdraw_amount } => try_arb_below_peg(deps, env, info, amount, uaust_withdraw_amount),
        HandleMsg::ProvideLiquidity{ asset } => try_provide_liquidity(deps, info, asset),
        HandleMsg::AnchorDeposit{ amount } => try_deposit_to_anchor(deps, info, amount),
        HandleMsg::SetSlippage{ slippage } => set_slippage(deps, info, slippage),
        HandleMsg::SetBurnAddress{ burn_addr } => set_burn_addr(deps, info, burn_addr)
    }
}
pub fn try_withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    sender: String,
    amount: Uint128,
) -> VaultResult {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;

    let total_share: Uint128 = query_supply(&deps.querier, deps.api.addr_humanize(&info.liquidity_token)?)?;
    let total_deposits: Uint128 = compute_total_deposits(deps.as_ref(), &info)?;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);
    let refund_asset: Asset = Asset{
        info: AssetInfo::NativeToken{ denom: get_stable_denom(deps.as_ref())? },
        amount: total_deposits * share_ratio
    };

    let mut response = Response::new()
        .add_attribute("action", "withdraw_liquidity")
        .add_attribute("withdrawn_share", amount.to_string())
        .add_attribute("refund_asset", format!(" {}", refund_asset));
    // withdraw from anchor if necessary
    // TODO: Improve
    let state = STATE.load(deps.storage)?;
    let stable_balance: Uint128 = query_balance(&deps.querier, env.contract.address.clone(), get_stable_denom(deps.as_ref())?)?;
    if refund_asset.amount*Decimal::from_ratio(Uint128::from(50u64), Uint128::from(1u64)) > stable_balance {
        let uaust_amount: Uint128 = query_token_balance(&deps.querier, deps.api.addr_humanize(&state.aust_address)?, env.contract.address)?;
        let uaust_exchange_rate_response = query_aust_exchange_rate(deps.as_ref(), deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string())?;
        let uaust_ust_rate = Decimal::from_str(&uaust_exchange_rate_response.exchange_rate.to_string())?;
        let uaust_amount_in_uust = uaust_amount * uaust_ust_rate;
        // TODO: Improve
        if uaust_amount_in_uust > Uint128::from(10u64 * 1000000u64) || amount == total_share {
            response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
                contract_addr: state.aust_address.to_string(),
                msg: to_binary(
                    &Cw20ExecuteMsg::Send{
                        contract: state.anchor_money_market_address.to_string(),
                        amount: uaust_amount,
                        msg: to_binary(&AnchorMsg::RedeemStable{})?
                    }
                )?,
                funds: vec![]
            }));
        }
    }

    let refund_msg = match &refund_asset.info {
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer { recipient: sender, amount })?,
            funds: vec![],
        }),
        AssetInfo::NativeToken { .. } => CosmosMsg::Bank(BankMsg::Send {
            to_address: sender,
            amount: vec![refund_asset.deduct_tax(&deps.querier)?],
        }),
    };
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&info.liquidity_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        funds: vec![],
    });

    Ok(response.add_message(refund_msg).add_message(burn_msg))
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
    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: deps.api.addr_humanize(&state.profit_check_address)?.to_string(),
        msg: to_binary(
            &ProfitCheckMsg::BeforeTrade{}
        )?,
        funds: vec![]
    }))
    .add_message(swap_msg)
    .add_message(terraswap_msg)
    .add_submessage(SubMsg{
        msg: CosmosMsg::Wasm(WasmMsg::Execute{
            contract_addr: deps.api.addr_humanize(&state.profit_check_address)?.to_string(),
            msg: to_binary(
                &ProfitCheckMsg::AfterTrade{}
            )?,
            funds: vec![]
        }),
        gas_limit: None,
        id: TRADE_REPLY_ID,
        reply_on: ReplyOn::Success,
    });

    Ok(response)
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
    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: deps.api.addr_humanize(&state.profit_check_address)?.to_string(),
        msg: to_binary(
            &ProfitCheckMsg::BeforeTrade{}
        )?,
        funds: vec![]
    }))
    .add_message(terraswap_msg)
    .add_message(swap_msg)
    .add_submessage(SubMsg{
        msg: CosmosMsg::Wasm(WasmMsg::Execute{
            contract_addr: deps.api.addr_humanize(&state.profit_check_address)?.to_string(),
            msg: to_binary(
                &ProfitCheckMsg::AfterTrade{}
            )?,
            funds: vec![]
        }),
        gas_limit: None,
        id: TRADE_REPLY_ID,
        reply_on: ReplyOn::Success,
    });

    Ok(response)
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
    if msg.id == TRADE_REPLY_ID {
        let state = STATE.load(deps.storage)?;
        let response: LastProfitResponse = deps.querier.query_wasm_smart(deps.api.addr_humanize(&state.profit_check_address)?, &ProfitCheckQueryMsg::LastProfit{})?;
        let profit_share = response.last_profit * state.profit_burn_ratio;
        if profit_share == Uint128::zero() {
            return Err(StdError::generic_err(format!("profit share {} {} {}", profit_share, response.last_profit, state.profit_burn_ratio)));
        }
        return Ok(Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute{
            contract_addr: deps.api.addr_humanize(&state.burn_addr)?.to_string(),
            funds: vec![Coin{ denom: UST_DENOM.to_string(), amount: profit_share }],
            msg: to_binary(&BurnMsg::Deposit{})?
        })));
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

pub fn compute_total_deposits(
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
    asset.assert_sent_native_token_balance(&msg_info)?;

    let deposit: Uint128 = asset.amount;
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let total_deposits_in_ust: Uint128 = compute_total_deposits(deps.as_ref(), &info)? - deposit;

    let total_share = query_supply(&deps.querier, deps.api.addr_humanize(&info.liquidity_token)?)?;
    let share = if total_share == Uint128::zero() {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_ust)
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
    Ok(Response::new().add_message(msg))
}

pub fn try_deposit_to_anchor(
    deps: DepsMut,
    msg_info: MessageInfo,
    amount: Coin
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.owner {
        return Err(StableVaultError::Unauthorized{});
    }
    
    Ok(try_deposit(deps.api.addr_humanize(&state.anchor_money_market_address)?.to_string(), amount)?)
}

pub fn set_slippage(
    deps: DepsMut,
    msg_info: MessageInfo,
    slippage: Decimal
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.owner {
        return Err(StableVaultError::Unauthorized{});
    }
    let mut info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    info.slippage = slippage;
    POOL_INFO.save(deps.storage, &info)?;
    Ok(Response::default())
}

pub fn set_burn_addr(
    deps: DepsMut,
    msg_info: MessageInfo,
    burn_addr: String
) -> StdResult<Response<TerraMsgWrapper>> {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.owner {
        return Err(StdError::generic_err("Unauthorized."));
    }
    let mut state = STATE.load(deps.storage)?;
    state.burn_addr = deps.api.addr_canonicalize(&burn_addr)?;
    STATE.save(deps.storage, &state)?;
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

    let total_deposits_in_ust = compute_total_deposits(deps, &info)?;

    Ok(PoolResponse { assets, total_deposits_in_ust, total_share })
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
            profit_burn_ratio: Decimal::percent(10u64),
            asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() },
            slippage: Decimal::percent(1u64), token_code_id: 0u64
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

        let msg = HandleMsg::SetSlippage {
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

        let msg = HandleMsg::BelowPeg {
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

        let msg = HandleMsg::AbovePeg {
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
