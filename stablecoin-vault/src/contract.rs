use cosmwasm_std::{
    log, from_binary, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg, Uint128, Decimal
};
use terra_cosmwasm::{create_swap_msg, TerraMsgWrapper};

use cw20::{Cw20HandleMsg, Cw20ReceiveMsg, MinterResponse};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, PoolResponse, create_terraswap_msg, create_assert_limit_order_msg, AnchorMsg};
use crate::state::{config, State, LUNA_DENOM, read_pool_info, store_pool_info, AUST, ANCHOR, BURN_MINT_CONTRACT};
use crate::asset::{Asset, AssetInfo, PoolInfo, PoolInfoRaw};
use crate::hook::InitHook;
use crate::token::InitMsg as TokenInitMsg;
use crate::querier::{query_balance, query_token_balance, query_supply, query_aust_exchange_rate};
use crate::pair::Cw20HookMsg;
use std::str::FromStr;


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
        trader: deps.api.canonical_address(&env.message.sender)?,
        pool_address: deps.api.canonical_address(&msg.pool_address)?,
    };

    config(&mut deps.storage).save(&state)?;

    let pool_info: &PoolInfoRaw = &PoolInfoRaw {
        contract_addr: deps.api.canonical_address(&env.contract.address)?,
        liquidity_token: CanonicalAddr::default(),
        asset_infos: [
            msg.asset_info.to_raw(&deps)?,
            AssetInfo::NativeToken{ denom: LUNA_DENOM.to_string()}.to_raw(&deps)?,
            AssetInfo::Token{ contract_addr: HumanAddr::from(AUST) }.to_raw(&deps)?
        ],
    };

    store_pool_info(&mut deps.storage, &pool_info)?;


    // Create LP token
    let messages: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: msg.token_code_id,
        msg: to_binary(&TokenInitMsg {
            name: "test liquidity token".to_string(),
            symbol: "tLP".to_string(),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: env.contract.address.clone(),
                cap: None,
            }),
            init_hook: Some(InitHook {
                msg: to_binary(&HandleMsg::PostInitialize {})?,
                contract_addr: env.contract.address,
            }),
        })?,
        send: vec![],
        label: None,
    })];

    // if let Some(hook) = msg.init_hook {
    //     messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: hook.contract_addr,
    //         msg: hook.msg,
    //         send: vec![],
    //     }));
    // }

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    match msg {
        HandleMsg::Receive(msg) => receive_cw20(deps, env, msg),
        HandleMsg::AbovePeg { amount, luna_price, residual_luna } => try_arb_above_peg(deps, env, amount, luna_price, residual_luna),
        HandleMsg::BelowPeg { amount, luna_price, residual_luna } => try_arb_below_peg(deps, env, amount, luna_price, residual_luna),
        HandleMsg::PostInitialize{ } => try_post_initialize(deps, env),
        HandleMsg::ProvideLiquidity{ asset } => try_provide_liquidity(deps, env, asset),
        HandleMsg::AnchorDeposit{ amount } => try_deposit_to_anchor(deps, env, amount),
        HandleMsg::AnchorWithdraw{ amount } => try_withdrawal_from_anchor(deps, env, amount),
    }
}

pub fn to_luna(coin: Coin, luna_price: Coin) -> Coin {
    Coin{ denom: coin.denom, amount: coin.amount.clone().multiply_ratio(1u128, luna_price.amount.0) }
}

pub fn try_withdraw_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let info: PoolInfoRaw = read_pool_info(&deps.storage)?;
    let liquidity_addr: HumanAddr = deps.api.human_address(&info.liquidity_token)?;

    // FIX
    let pool: Asset = info.query_pools(&deps, &env.contract.address)?[0].clone();
    let total_share: Uint128 = query_supply(&deps, &liquidity_addr)?;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);
    let refund_asset: Asset = Asset{
        info: pool.info.clone(),
        amount: pool.amount * share_ratio
    };

    let refund_msg = match &refund_asset.info {
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_binary(&Cw20HandleMsg::Transfer { recipient: sender, amount: amount })?,
            send: vec![],
        }),
        AssetInfo::NativeToken { .. } => CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address,
            to_address: sender,
            amount: vec![refund_asset.deduct_tax(deps)?],
        }),
    };

    // update pool info
    Ok(HandleResponse {
        messages: vec![
            refund_msg,
            // burn liquidity token
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&info.liquidity_token)?,
                msg: to_binary(&Cw20HandleMsg::Burn { amount })?,
                send: vec![],
            }),
        ],
        log: vec![
            log("action", "withdraw_liquidity"),
            log("withdrawn_share", &amount.to_string()),
            log(
                "refund_asset",
                format!(" {}", refund_asset),
            ),
        ],
        data: None,
    })
}

pub fn receive_cw20<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            Cw20HookMsg::Swap {
                ..
            } => {
                Err(StdError::generic_err("no swaps can be performed in this pool"))
            }
            Cw20HookMsg::WithdrawLiquidity {} => {
                let config: PoolInfoRaw = read_pool_info(&deps.storage)?;
                if deps.api.canonical_address(&env.message.sender)? != config.liquidity_token {
                    return Err(StdError::unauthorized());
                }

                try_withdraw_liquidity(deps, env, cw20_msg.sender, cw20_msg.amount)
            }
        }
    } else {
        Err(StdError::generic_err("data should be given"))
    }
}

pub fn try_arb_below_peg<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Coin,
    luna_price: Coin,
    residual_luna: Uint128
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let state = config(&mut deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.trader {
        return Err(StdError::generic_err("Unauthorized."));
    }

    let ask_denom = LUNA_DENOM.to_string();

    let expected_luna_amount = amount.amount * Decimal::from_ratio(Uint128(1000000), luna_price.amount);
    // let assert_limit_order_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: HumanAddr::from(BURN_MINT_CONTRACT),
    //     send: vec![],
    //     msg: to_binary(&create_assert_limit_order_msg(amount.clone(), ask_denom.clone(), expected_luna_amount))?,
    // });
    let swap_msg = create_swap_msg(
        env.contract.address,
        amount.clone(),
        ask_denom.clone(),
    );
    let offer_coin = Coin{ denom: ask_denom.clone(), amount: residual_luna + expected_luna_amount};
    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&state.pool_address)?,
        send: vec![offer_coin.clone()],
        msg: to_binary(&create_terraswap_msg(offer_coin.clone(), Decimal::from_ratio(luna_price.amount, Uint128(1000000))))?,
    });

    Ok(HandleResponse {
        messages: vec![swap_msg, terraswap_msg],
        log: vec![],
        data: None,
    })
}

pub fn try_arb_above_peg<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Coin,
    luna_price: Coin,
    residual_luna: Uint128
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let state = config(&mut deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.trader {
        return Err(StdError::generic_err("Unauthorized."));
    }

    let ask_denom = LUNA_DENOM.to_string();

    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&state.pool_address)?,
        send: vec![amount.clone()],
        msg: to_binary(&create_terraswap_msg(amount.clone(), Decimal::from_ratio(luna_price.amount, Uint128(1000000))))?,
    });

    let offer_coin = Coin{ denom: ask_denom.clone(), amount: residual_luna + amount.amount * Decimal::from_ratio(Uint128(1000000), luna_price.amount)};
    // let min_stable_amount = amount.amount;
    // let assert_limit_order_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: HumanAddr::from(BURN_MINT_CONTRACT),
    //     send: vec![],
    //     msg: to_binary(&create_assert_limit_order_msg(offer_coin.clone(), amount.denom.clone(), min_stable_amount))?,
    // });
    let swap_msg = create_swap_msg(
        env.contract.address,
        offer_coin,
        amount.denom,
    );

    Ok(HandleResponse {
        messages: vec![terraswap_msg, swap_msg],
        log: vec![],
        data: None,
    })
}

pub fn try_send_funds(
    env: Env,
    amount: Coin,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let msg = CosmosMsg::Bank(BankMsg::Send{
        from_address: env.contract.address,
        to_address: env.message.sender,
        amount: vec![amount]
    });

    Ok(HandleResponse {
        messages: vec![msg],
        log: vec![],
        data: None,
    })
}

pub fn try_post_initialize<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let config: PoolInfoRaw = read_pool_info(&deps.storage)?;

    // permission check
    if config.liquidity_token != CanonicalAddr::default() {
        return Err(StdError::unauthorized());
    }

    store_pool_info(
        &mut deps.storage,
        &PoolInfoRaw {
            liquidity_token: deps.api.canonical_address(&env.message.sender)?,
            ..config
        },
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("liquidity_token_addr", env.message.sender.as_str())],
        data: None,
    })
}

pub fn compute_total_deposits<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    contract_address: &HumanAddr,
    info: &PoolInfoRaw
) -> StdResult<Uint128> {
    // assert slippage tolerance
    // assert_slippage_tolerance(&slippage_tolerance, &deposits, &pools)?;

    let ust_info = info.asset_infos[0].to_normal(deps)?;
    let ust_amount = match ust_info {
        AssetInfo::Token{..} => Uint128(0),
        AssetInfo::NativeToken{denom} => query_balance(deps, &contract_address, denom)?
    };

    // let luna_info = info.asset_infos[1].to_normal(deps)?;


    let aust_info = info.asset_infos[2].to_normal(deps)?;
    let aust_amount = match aust_info {
        AssetInfo::Token{contract_addr} => query_token_balance(deps, &contract_addr, &contract_address)?,
        AssetInfo::NativeToken{..} => Uint128(0)
    };

    let epoch_state_response = query_aust_exchange_rate(deps)?;
    let aust_exchange_rate= Decimal::from_str(&epoch_state_response.exchange_rate.to_string())?;
    let aust_value_in_ust = aust_exchange_rate*aust_amount;

    let total_deposits_in_ust = ust_amount + aust_value_in_ust;
    Ok(total_deposits_in_ust)
}

pub fn try_provide_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: Asset
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    asset.assert_sent_native_token_balance(&env)?;

    let deposit: Uint128 = asset.amount;
    let info: PoolInfoRaw = read_pool_info(&deps.storage)?;
    let total_deposits_in_ust: Uint128 = (compute_total_deposits(deps, &env.contract.address, &info)? - deposit)?;

    let liquidity_token = deps.api.human_address(&info.liquidity_token)?;
    let total_share = query_supply(&deps, &liquidity_token)?;
    let share = if total_share == Uint128::zero() {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_ust)
    };


    let mut response = HandleResponse::<TerraMsgWrapper>::default();
    // mint LP token to sender
    response.messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&info.liquidity_token)?,
        msg: to_binary(&Cw20HandleMsg::Mint {
            recipient: env.message.sender,
            amount: share,
        })?,
        send: vec![],
    }));
    Ok(response)
}

pub fn try_deposit_to_anchor<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Coin
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    if amount.denom != "uusd" {
        return Err(StdError::generic_err("Wrong currency. Only UST (denom: uusd) is supported."));
    }

    let state = config(&mut deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.trader {
        return Err(StdError::generic_err("Unauthorized."));
    }

    let mut response = HandleResponse::default();
    response.messages.push(CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: HumanAddr::from(ANCHOR),
        msg: to_binary(&AnchorMsg::DepositStable{})?,
        send: vec![amount]
    }));

    Ok(response)
}

pub fn try_withdrawal_from_anchor<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let state = config(&mut deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.trader {
        return Err(StdError::generic_err("Unauthorized."));
    }

    let mut response = HandleResponse::default();
    response.messages.push(CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: HumanAddr::from(AUST),
        msg: to_binary(
            &AnchorMsg::Send{
                contract: HumanAddr::from(ANCHOR),
                amount: amount,
                msg: to_binary(&AnchorMsg::RedeemStable{})?
            }
        )?,
        send: vec![]
    }));

    Ok(response)
}


pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Asset{} => to_binary(&try_query_asset(deps)?),
        QueryMsg::Pool{} => to_binary(&try_query_pool(deps)?)
    }
}

pub fn try_query_asset<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<PoolInfo> {

    let info: PoolInfoRaw = read_pool_info(&deps.storage)?;
    info.to_normal(&deps)
}

pub fn try_query_pool<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<PoolResponse> {
    let info: PoolInfoRaw = read_pool_info(&deps.storage)?;
    let contract_addr = deps.api.human_address(&info.contract_addr)?;
    let assets: [Asset; 3] = info.query_pools(&deps, &contract_addr)?;
    let total_share: Uint128 =
        query_supply(&deps, &deps.api.human_address(&info.liquidity_token)?)?;

    let resp = PoolResponse {
        assets,
        total_share,
    };

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, HumanAddr, Uint128};
    use terra_cosmwasm::TerraRoute;
    use crate::asset::AssetInfo;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { pool_address: HumanAddr::from("test pool"), token_code_id: 0u64, asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() } };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn when_given_a_below_peg_msg_then_handle_returns_first_a_mint_then_a_terraswap_msg() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { pool_address: HumanAddr::from("test pool"), token_code_id: 0u64, asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() } };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = init(&mut deps, env, msg).unwrap();

        let msg = HandleMsg::BelowPeg {
            amount: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
            luna_price: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
            residual_luna: Uint128(0),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(2, res.messages.len());
        let first_msg = res.messages[0].clone();
        match first_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
            CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
            CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected")
        }
        let second_msg = res.messages[1].clone();
        match second_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(_t) => panic!("unexpected"),
            CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
            CosmosMsg::Wasm(_wasm_msg) => {}
        }
    }

    #[test]
    fn when_given_an_above_peg_msg_then_handle_returns_first_a_terraswap_then_a_mint_msg() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { pool_address: HumanAddr::from("test pool"), token_code_id: 0u64, asset_info: AssetInfo::NativeToken{ denom: "uusd".to_string() } };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = init(&mut deps, env, msg).unwrap();

        let msg = HandleMsg::AbovePeg {
            amount: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
            luna_price: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
            residual_luna: Uint128(0),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(2, res.messages.len());
        let first_msg = res.messages[0].clone();
        match first_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(_t) => panic!("unexpected"),
            CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
            CosmosMsg::Wasm(_wasm_msg) => {}
        }
        let second_msg = res.messages[1].clone();
        match second_msg {
            CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
            CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
            CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
            CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected")
        }
    }
}
