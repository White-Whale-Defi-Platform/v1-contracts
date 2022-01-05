use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdError, StdResult, Uint128, WasmQuery, CosmosMsg, Empty, WasmMsg,
};

use crate::error::ProfitCheckError;
use crate::state::{State, ADMIN, CONFIG};
use white_whale::profit_check::msg::{
    ExecuteMsg, InstantiateMsg, LastBalanceResponse, LastProfitResponse, QueryMsg, VaultResponse,
};
use white_whale::ust_vault::msg::{ValueResponse, VaultQueryMsg, ExecuteMsg::SendWarchestCommission as VaultCommissionMsg};
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
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        vault_address: deps.api.addr_canonicalize(&msg.vault_address.to_string())?,
        denom: msg.denom,
        last_balance: Uint128::zero(),
        last_profit: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ProfitCheckResult {
    match msg {
        ExecuteMsg::AfterTrade { loan_fee } => after_trade(deps, info, loan_fee),
        ExecuteMsg::BeforeTrade {} => before_trade(deps, info),
        ExecuteMsg::SetVault { vault_address } => set_vault_address(deps, info, vault_address),
    }
}

// Resets last trade and sets current UST balance of caller
pub fn before_trade(deps: DepsMut, info: MessageInfo) -> ProfitCheckResult {
    let mut conf = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != conf.vault_address {
        return Err(ProfitCheckError::Std(StdError::generic_err("Unauthorized")));
    }

    // last_balance call can not be reset until after the loan.
    if conf.last_balance != Uint128::zero() {
        return Err(ProfitCheckError::Nonzero {});
    }

    conf.last_profit = Uint128::zero();

    conf.last_balance = get_vault_value(deps.as_ref())?;
    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::default().add_attribute("value before trade: ", conf.last_balance.to_string()))
}

// Checks if balance increased after the trade
pub fn after_trade(deps: DepsMut, info: MessageInfo, loan_fee: Uint128) -> ProfitCheckResult {
    let mut conf = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != conf.vault_address {
        return Err(ProfitCheckError::Std(StdError::generic_err("Unauthorized")));
    }

    let balance = get_vault_value(deps.as_ref())?;

    if balance < conf.last_balance + loan_fee {
        return Err(ProfitCheckError::CancelLosingTrade {});
    }

    let profit = balance - conf.last_balance;

    conf.last_profit = profit;
    conf.last_balance = Uint128::zero();
    CONFIG.save(deps.storage, &conf)?;

    // Create commission message
    let commission_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&conf.vault_address)?.into_string(),
        msg: to_binary(&VaultCommissionMsg {
            profit
        })?,
        funds: vec![],
    });

    Ok(Response::default().add_attribute("value after trade: ", balance.to_string()).add_message(commission_msg))
}

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

pub fn get_vault_value(deps: Deps) -> StdResult<Uint128> {
    let config = CONFIG.load(deps.storage)?;
    let response: ValueResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps.api.addr_humanize(&config.vault_address)?.to_string(),
        msg: to_binary(&VaultQueryMsg::VaultValue {})?,
    }))?;
    Ok(response.total_ust_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::mock_dependencies;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{from_binary, Api, Coin};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InstantiateMsg {
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
        let msg = InstantiateMsg {
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
            ExecuteMsg::SetVault {
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
        let msg = InstantiateMsg {
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
            ExecuteMsg::BeforeTrade {},
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
            ExecuteMsg::AfterTrade {
                loan_fee: Uint128::new(1000000),
            },
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
        let msg = InstantiateMsg {
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
            ExecuteMsg::BeforeTrade {},
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse =
            from_binary(&query(deps.as_ref(), env.clone(), QueryMsg::LastBalance {}).unwrap())
                .unwrap();
        assert_eq!(res.last_balance, initial_balance);

        let res = execute(
            deps.as_mut(),
            env,
            vault_info,
            ExecuteMsg::AfterTrade {
                loan_fee: Uint128::zero(),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len())
    }

    #[test]
    fn test_check_before_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InstantiateMsg {
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

        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::BeforeTrade {});
        match res {
            Err(..) => {}
            _ => panic!("unexpected"),
        }

        let vault_info = MessageInfo {
            sender: vault_address.clone(),
            funds: vec![],
        };
        let _res = execute(deps.as_mut(), env, vault_info, ExecuteMsg::BeforeTrade {}).unwrap();
    }

    #[test]
    fn test_check_after_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InstantiateMsg {
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

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::AfterTrade {
                loan_fee: Uint128::new(1000000),
            },
        );
        match res {
            Err(..) => {}
            _ => panic!("unexpected"),
        }

        let vault_info = MessageInfo {
            sender: vault_address.clone(),
            funds: vec![],
        };
        // TODO: this throws error as vault_address is not in the deps. Whats the best way to solve this?
        let _res = execute(
            deps.as_mut(),
            env,
            vault_info,
            ExecuteMsg::AfterTrade {
                loan_fee: Uint128::zero(),
            },
        )
        .unwrap();
    }

    #[test]
    fn test_only_owner_can_change_vault() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let other_vault_address = deps.api.addr_validate("other_test_vault").unwrap();
        let msg = InstantiateMsg {
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
            ExecuteMsg::SetVault {
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
