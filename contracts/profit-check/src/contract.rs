use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use white_whale::query::anchor::query_aust_exchange_rate;
use terraswap::querier::{query_balance, query_token_balance};
use terraswap::asset::{Asset, AssetInfo};


use crate::error::ProfitCheckError;
use crate::state::{State, ADMIN, CONFIG};
use white_whale::profit_check::msg::{
    HandleMsg, InitMsg, LastBalanceResponse, LastProfitResponse, QueryMsg, VaultResponse,
};
/*
    Profit check is used by the ust vault to see if a proposed trade is indeed profitable.
    before_trade is called before the trade to set the account balance
    after_trade is called after the trade and checks weather a profit was made
    If the balance of the contract is smaller after the trade, an error gets thrown which resets the contract state to
    the state before the contract call.
*/
type ProfitCheckResult = Result<Response, ProfitCheckError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let state = State {
        vault_address: deps.api.addr_canonicalize(&msg.vault_address.to_string())?,
        anchor_money_market_address: deps.api.addr_canonicalize(&msg.vault_address.to_string())?,
        aust_address: deps.api.addr_canonicalize(&msg.aust_address)?,
        denom: msg.denom,
        last_balance: Uint128::zero(),
        last_profit: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: HandleMsg) -> ProfitCheckResult {
    match msg {
        HandleMsg::AfterTrade {} => after_trade(deps, info),
        HandleMsg::BeforeTrade {} => before_trade(deps, info),
        HandleMsg::SetVault { vault_address } => set_vault_address(deps, info, vault_address),
    }
}

// Resets last trade and sets current UST balance of caller
pub fn before_trade(deps: DepsMut, info: MessageInfo) -> ProfitCheckResult {
    let mut conf = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != conf.vault_address {
        return Err(ProfitCheckError::Std(StdError::generic_err("Unauthorized")));
    }

    if conf.last_balance != Uint128::zero() {
        return Err(ProfitCheckError::Std(StdError::generic_err("Nonzero")));
    }

    conf.last_profit = Uint128::zero();

    let aust_exchange_rate = query_aust_exchange_rate(
        deps.as_ref(),
        deps.api
            .addr_humanize(&conf.anchor_money_market_address)?
            .to_string(),
    )?;

    let aust_balance = query_token_balance(
        &deps.querier,
        info.sender.clone(),
        deps.api.addr_humanize(&conf.aust_address)?,
    )?;
    
    let ust_balance = query_balance(&deps.querier, info.sender, conf.denom.clone())?;

    conf.last_balance = ust_balance + aust_balance * aust_exchange_rate;
    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::default())
}

// Checks if balance increased after the trade
pub fn after_trade(deps: DepsMut, info: MessageInfo) -> ProfitCheckResult {
    let mut conf = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != conf.vault_address {
        return Err(ProfitCheckError::Std(StdError::generic_err("Unauthorized")));
    }

    let aust_exchange_rate = query_aust_exchange_rate(
        deps.as_ref(),
        deps.api
            .addr_humanize(&conf.anchor_money_market_address)?
            .to_string(),
    )?;

    let aust_balance = query_token_balance(
        &deps.querier,
        info.sender.clone(),
        deps.api.addr_humanize(&conf.aust_address)?,
    )?;
    
    let ust_balance = query_balance(&deps.querier, info.sender, conf.denom.clone())?;
    let balance = ust_balance + aust_balance * aust_exchange_rate;

    if balance < conf.last_balance {
        return Err(ProfitCheckError::CancelLosingTrade {});
    }

    conf.last_profit = balance - conf.last_balance;
    conf.last_balance = Uint128::zero();
    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::default())
}

// compute total value of deposits in UST and return
// pub fn compute_total_value(
//     deps: Deps,
//     info: &PoolInfoRaw,
// ) -> StdResult<(Uint128, Uint128, Uint128)> {
//     let state = STATE.load(deps.storage)?;
//     let stable_info = info.asset_infos[0].to_normal(deps.api)?;
//     let stable_denom = match stable_info {
//         AssetInfo::Token { .. } => String::default(),
//         AssetInfo::NativeToken { denom } => denom,
//     };
//     let stable_amount = query_balance(&deps.querier, info.contract_addr.clone(), stable_denom)?;

//     let aust_info = info.asset_infos[2].to_normal(deps.api)?;
//     let aust_amount = aust_info.query_pool(&deps.querier, deps.api, info.contract_addr.clone())?;
//     let aust_exchange_rate = query_aust_exchange_rate(
//         deps,
//         deps.api
//             .addr_humanize(&state.anchor_money_market_address)?
//             .to_string(),
//     )?;

//     let aust_value_in_ust = aust_exchange_rate * aust_amount;

//     let total_deposits_in_ust = stable_amount + aust_value_in_ust;
//     Ok((total_deposits_in_ust, stable_amount, aust_value_in_ust))
// }

// Set address of UST vault
pub fn set_vault_address(
    deps: DepsMut,
    info: MessageInfo,
    vault_address: String,
) -> ProfitCheckResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut conf = CONFIG.load(deps.storage)?;
    conf.vault_address = deps.api.addr_canonicalize(&vault_address)?;
    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastBalance {} => to_binary(&try_query_last_balance(deps)?),
        QueryMsg::LastProfit {} => to_binary(&try_query_last_profit(deps)?),
        QueryMsg::Vault {} => to_binary(&try_query_vault_address(deps)?),
    }
}

pub fn try_query_last_profit(deps: Deps) -> StdResult<LastProfitResponse> {
    let conf = CONFIG.load(deps.storage)?;
    Ok(LastProfitResponse {
        last_profit: conf.last_profit,
    })
}

pub fn try_query_last_balance(deps: Deps) -> StdResult<LastBalanceResponse> {
    let conf = CONFIG.load(deps.storage)?;
    Ok(LastBalanceResponse {
        last_balance: conf.last_balance,
    })
}

pub fn try_query_vault_address(deps: Deps) -> StdResult<VaultResponse> {
    let conf = CONFIG.load(deps.storage)?;
    Ok(VaultResponse {
        vault_address: deps.api.addr_humanize(&conf.vault_address)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{from_binary, Api, Coin};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string(),
        };
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse =
            from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::LastBalance {}).unwrap())
                .unwrap();
        assert_eq!(res.last_balance, Uint128::zero());

        let res: VaultResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::Vault {}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);
    }

    #[test]
    fn test_set_vault() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string(),
        };
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res: VaultResponse =
            from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::Vault {}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);

        let other_vault = deps.api.addr_validate("test_vault").unwrap();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            HandleMsg::SetVault {
                vault_address: other_vault.to_string(),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        let res: VaultResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::Vault {}).unwrap()).unwrap();
        assert_eq!(res.vault_address, other_vault);
    }

    #[test]
    fn test_failure_of_profit_check() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string(),
        };
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let initial_balance = Uint128::from(100u64);
        deps.querier.update_balance(
            vault_address.clone(),
            vec![Coin {
                denom: msg.denom.clone(),
                amount: initial_balance,
            }],
        );

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let vault_info = MessageInfo {
            sender: vault_address.clone(),
            funds: vec![],
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            vault_info.clone(),
            HandleMsg::BeforeTrade {},
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse =
            from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::LastBalance {}).unwrap())
                .unwrap();
        assert_eq!(res.last_balance, initial_balance);

        deps.querier.update_balance(
            vault_address,
            vec![Coin {
                denom: msg.denom,
                amount: Uint128::from(99u64),
            }],
        );

        let res = execute(
            deps.as_mut(),
            env.clone(),
            vault_info,
            HandleMsg::AfterTrade {},
        );
        match res {
            Err(..) => {}
            _ => panic!("unexpected"),
        }

        let res: LastBalanceResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::LastBalance {}).unwrap()).unwrap();
        assert_eq!(res.last_balance, initial_balance);
    }

    #[test]
    fn test_success_of_profit_check() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string(),
        };
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let initial_balance = Uint128::from(100u64);
        deps.querier.update_balance(
            vault_address.clone(),
            vec![Coin {
                denom: msg.denom.clone(),
                amount: initial_balance,
            }],
        );

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let vault_info = MessageInfo {
            sender: vault_address.clone(),
            funds: vec![],
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            vault_info.clone(),
            HandleMsg::BeforeTrade {},
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse =
            from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::LastBalance {}).unwrap())
                .unwrap();
        assert_eq!(res.last_balance, initial_balance);

        let res = execute(deps.as_mut(), env, vault_info, HandleMsg::AfterTrade {}).unwrap();
        assert_eq!(0, res.messages.len())
    }

    #[test]
    fn test_check_before_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string(),
        };
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let res = execute(deps.as_mut(), env.clone(), info, HandleMsg::BeforeTrade {});
        match res {
            Err(..) => {}
            _ => panic!("unexpected"),
        }

        let vault_info = MessageInfo {
            sender: vault_address.clone(),
            funds: vec![],
        };
        let _res = execute(deps.as_mut(), env, vault_info, HandleMsg::BeforeTrade {}).unwrap();
    }

    #[test]
    fn test_check_after_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string(),
        };
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let res = execute(deps.as_mut(), env.clone(), info, HandleMsg::AfterTrade {});
        match res {
            Err(..) => {}
            _ => panic!("unexpected"),
        }

        let vault_info = MessageInfo {
            sender: vault_address.clone(),
            funds: vec![],
        };
        let _res = execute(deps.as_mut(), env, vault_info, HandleMsg::AfterTrade {}).unwrap();
    }

    #[test]
    fn test_only_owner_can_change_vault() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let other_vault_address = deps.api.addr_validate("other_test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string(),
        };
        let env = mock_env();
        let owner_info = MessageInfo {
            sender: deps.api.addr_validate("owner").unwrap(),
            funds: vec![],
        };
        let user_info = MessageInfo {
            sender: deps.api.addr_validate("user").unwrap(),
            funds: vec![],
        };

        let _res =
            instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg.clone()).unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            user_info,
            HandleMsg::SetVault {
                vault_address: other_vault_address.to_string(),
            },
        );
        match res {
            Err(..) => {}
            _ => panic!("unexpected"),
        }

        let res: VaultResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::Vault {}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);
    }
}
