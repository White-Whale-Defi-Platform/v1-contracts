use cosmwasm_std::{
    log, from_binary, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg, Uint128, Decimal
};
use terra_cosmwasm::{TerraMsgWrapper};
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw};
use terraswap::pair::{Cw20HookMsg};
use terraswap::querier::{query_balance, query_token_balance, query_supply};
use terraswap::hook::InitHook;
use terraswap::token::InitMsg as TokenInitMsg;

use cw20::{Cw20HandleMsg, Cw20ReceiveMsg, MinterResponse};

use crate::pool_info::{PoolInfo as PairInfo, PoolInfoRaw as PairInfoRaw};
use crate::msg::{HandleMsg, InitMsg, PoolResponse};
use crate::state::{config, config_read, State, LUNA_DENOM, read_pool_info, store_pool_info};
use white_whale::msg::{create_terraswap_msg, VaultQueryMsg as QueryMsg};
use white_whale::query::terraswap::simulate_swap as simulate_terraswap_swap;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
        trader: deps.api.canonical_address(&env.message.sender)?,
        pool_address: deps.api.canonical_address(&msg.pool_address)?,
        bluna_hub_address: deps.api.canonical_address(&msg.bluna_hub_address)?,
        bluna_address: deps.api.canonical_address(&msg.bluna_address)?,
    };

    config(&mut deps.storage).save(&state)?;

    let pool_info: &PairInfoRaw = &PairInfoRaw {
        contract_addr: deps.api.canonical_address(&env.contract.address)?,
        liquidity_token: CanonicalAddr::default(),
        slippage: msg.slippage,
        asset_infos: [
            AssetInfoRaw::Token{ contract_addr: deps.api.canonical_address(&msg.bluna_address)? },
            AssetInfo::NativeToken{ denom: LUNA_DENOM.to_string()}.to_raw(deps)?
        ],
    };
    store_pool_info(&mut deps.storage, pool_info)?;

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
        HandleMsg::Swap{ amount } => try_swap(deps, env, amount),
        HandleMsg::PostInitialize{ } => try_post_initialize(deps, env),
        HandleMsg::ProvideLiquidity{ asset } => try_provide_liquidity(deps, env, asset),
        HandleMsg::SetSlippage{ slippage } => set_slippage(deps, env, slippage),
    }
}

pub fn try_swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    offer_coin: Coin,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let state = config_read(&deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.trader {
        return Err(StdError::unauthorized());
    }

    let mut response = HandleResponse::default();
    let slippage = (read_pool_info(&deps.storage)?).slippage;
    let belief_price = Decimal::from_ratio(simulate_terraswap_swap(deps, deps.api.human_address(&state.pool_address)?, offer_coin.clone())?, offer_coin.amount);
    let msg = if offer_coin.denom == "uluna" {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&state.pool_address)?,
            send: vec![offer_coin.clone()],
            msg: to_binary(&create_terraswap_msg(offer_coin, belief_price, Some(slippage)))?,
        })
    } else {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&state.bluna_address)?,
            send: vec![],
            msg: to_binary(&Cw20HandleMsg::Send{
                contract: deps.api.human_address(&state.pool_address)?,
                amount: offer_coin.amount,
                msg: Some(to_binary(&create_terraswap_msg(offer_coin, belief_price, Some(slippage)))?)
            })?
        })
    };
    response.messages.push(msg);

    Ok(response)
}

pub fn compute_total_deposits<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    info: &PairInfoRaw
) -> StdResult<Uint128> {
    let state = config_read(&deps.storage).load()?;
    let contract_address = deps.api.human_address(&info.contract_addr)?;
    let deposits_in_luna = query_balance(deps, &contract_address, LUNA_DENOM.to_string())?;
    let deposits_in_bluna = query_token_balance(deps, &deps.api.human_address(&state.bluna_address)?, &contract_address)?;
    let total_deposits_in_luna = deposits_in_luna + deposits_in_bluna;
    Ok(total_deposits_in_luna)
}

pub fn try_withdraw_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let info: PairInfoRaw = read_pool_info(&deps.storage)?;
    let liquidity_addr: HumanAddr = deps.api.human_address(&info.liquidity_token)?;

    let total_share: Uint128 = query_supply(deps, &liquidity_addr)?;
    let total_deposits: Uint128 = compute_total_deposits(deps, &info)?;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);
    let refund_asset: Asset = Asset{
        info: AssetInfo::NativeToken{ denom: get_stable_denom(deps)? },
        amount: total_deposits * share_ratio
    };

    let mut response = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "withdraw_liquidity"),
            log("withdrawn_share", &amount.to_string()),
            log(
                "refund_asset",
                format!(" {}", refund_asset),
            ),
        ],
        data: None,
    };

    let refund_msg = match &refund_asset.info {
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_binary(&Cw20HandleMsg::Transfer { recipient: sender, amount })?,
            send: vec![],
        }),
        AssetInfo::NativeToken { .. } => CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address,
            to_address: sender,
            amount: vec![refund_asset.deduct_tax(deps)?],
        }),
    };
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&info.liquidity_token)?,
        msg: to_binary(&Cw20HandleMsg::Burn { amount })?,
        send: vec![],
    });
    response.messages.push(refund_msg);
    response.messages.push(burn_msg);

    Ok(response)
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
                let config: PairInfoRaw = read_pool_info(&deps.storage)?;
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

pub fn get_stable_denom<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<String> {
    let info: PairInfoRaw = read_pool_info(&deps.storage)?;
    let stable_info = info.asset_infos[0].to_normal(deps)?;
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
    Ok(Decimal::from_ratio((Uint128(100) - Uint128(100) * slippage)?, Uint128(100)))
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
    let config: PairInfoRaw = read_pool_info(&deps.storage)?;

    // permission check
    if config.liquidity_token != CanonicalAddr::default() {
        return Err(StdError::unauthorized());
    }

    store_pool_info(
        &mut deps.storage,
        &PairInfoRaw {
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

pub fn try_provide_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: Asset
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    asset.assert_sent_native_token_balance(&env)?;

    let deposit: Uint128 = asset.amount;
    let info: PairInfoRaw = read_pool_info(&deps.storage)?;
    let total_deposits_in_luna: Uint128 = (compute_total_deposits(deps, &info)? - deposit)?;

    let liquidity_token = deps.api.human_address(&info.liquidity_token)?;
    let total_share = query_supply(deps, &liquidity_token)?;
    let share = if total_share == Uint128::zero() {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_luna)
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

pub fn set_slippage<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    slippage: Decimal
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let state = config(&mut deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::unauthorized());
    }
    let mut info = read_pool_info(&deps.storage)?;
    info.slippage = slippage;
    store_pool_info(&mut deps.storage, &info)?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config{} => to_binary(&try_query_config(deps)?),
        QueryMsg::Pool{} => to_binary(&try_query_pool(deps)?)
    }
}

pub fn try_query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<PairInfo> {

    let info = read_pool_info(&deps.storage)?;
    info.to_normal(deps)
}

pub fn try_query_pool<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<PoolResponse> {
    let info = read_pool_info(&deps.storage)?;
    let contract_addr = deps.api.human_address(&info.contract_addr)?;
    let assets: [Asset; 2] = info.query_pools(deps, &contract_addr)?;
    let total_share: Uint128 =
        query_supply(deps, &deps.api.human_address(&info.liquidity_token)?)?;


    let resp = PoolResponse {
        assets,
        total_share,
    };

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, HumanAddr, Uint128};
    use terra_cosmwasm::TerraRoute;
    use terraswap::asset::AssetInfo;

    fn get_test_init_msg() -> InitMsg {
        InitMsg {
            pool_address: HumanAddr::from("test_pool"),
            bluna_hub_address: HumanAddr::from("test_mm"),
            bluna_address: HumanAddr::from("test_aust"),
            slippage: Decimal::percent(1u64), token_code_id: 0u64
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = get_test_init_msg();
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn proper_set_slippage() {
        let mut deps = mock_dependencies(20, &[]);

        let init_msg = get_test_init_msg();
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env.clone(), init_msg).unwrap();
        assert_eq!(1, res.messages.len());

        let state = read_pool_info(&deps.storage).unwrap();
        assert_eq!(state.slippage, Decimal::percent(1u64));

        let msg = HandleMsg::SetSlippage {
            slippage: Decimal::one()
        };
        let _res = handle(&mut deps, env, msg).unwrap();
        let state = read_pool_info(&deps.storage).unwrap();
        assert_eq!(state.slippage, Decimal::one());
    }

    // #[test]
    // fn when_given_a_below_peg_msg_then_handle_returns_first_a_mint_then_a_terraswap_msg() {
    //     let mut deps = mock_dependencies(20, &[]);

    //     let init_msg = get_test_init_msg();
    //     let env = mock_env("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let _res = init(&mut deps, env, init_msg).unwrap();

    //     let msg = HandleMsg::BelowPeg {
    //         amount: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
    //         uaust_withdraw_amount: Uint128(0)
    //     };
    //     let env = mock_env("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = handle(&mut deps, env, msg).unwrap();
    //     assert_eq!(2, res.messages.len());
    //     let first_msg = res.messages[0].clone();
    //     match first_msg {
    //         CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
    //         CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
    //         CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
    //         CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected")
    //     }
    //     let second_msg = res.messages[1].clone();
    //     match second_msg {
    //         CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
    //         CosmosMsg::Custom(_t) => panic!("unexpected"),
    //         CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
    //         CosmosMsg::Wasm(_wasm_msg) => {}
    //     }
    // }

    // #[test]
    // fn when_given_an_above_peg_msg_then_handle_returns_first_a_terraswap_then_a_mint_msg() {
    //     let mut deps = mock_dependencies(20, &[]);

    //     let init_msg = get_test_init_msg();
    //     let env = mock_env("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let _res = init(&mut deps, env, init_msg).unwrap();

    //     let msg = HandleMsg::AbovePeg {
    //         amount: Coin{denom: "uusd".to_string(), amount: Uint128(1000000)},
    //         uaust_withdraw_amount: Uint128(0)
    //     };
    //     let env = mock_env("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = handle(&mut deps, env, msg).unwrap();
    //     assert_eq!(2, res.messages.len());
    //     let first_msg = res.messages[0].clone();
    //     match first_msg {
    //         CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
    //         CosmosMsg::Custom(_t) => panic!("unexpected"),
    //         CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
    //         CosmosMsg::Wasm(_wasm_msg) => {}
    //     }
    //     // let second_msg = res.messages[1].clone();
    //     // match second_msg {
    //     //     CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
    //     //     CosmosMsg::Custom(_t) => panic!("unexpected"),
    //     //     CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
    //     //     CosmosMsg::Wasm(_wasm_msg) => {}
    //     // }
    //     let third_msg = res.messages[1].clone();
    //     match third_msg {
    //         CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
    //         CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
    //         CosmosMsg::Staking(_staking_msg) => panic!("unexpected"),
    //         CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected")
    //     }
    // }
}
