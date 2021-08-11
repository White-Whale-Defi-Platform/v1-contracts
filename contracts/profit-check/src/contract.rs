use cosmwasm_std::{ entry_point,
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128
};
use terraswap::querier::{query_balance};

use white_whale::profit_check::msg::{HandleMsg, InitMsg, QueryMsg, LastBalanceResponse, VaultResponse};
use crate::state::{CONFIG, State};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.addr_canonicalize(&info.sender.to_string())?,
        vault_address: deps.api.addr_canonicalize(&msg.vault_address.to_string())?,
        denom: msg.denom,
        last_balance: Uint128::zero()
    };

    CONFIG.save(deps.storage, &state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> StdResult<Response> {
    match msg {
        HandleMsg::AfterTrade{} => after_trade(deps, info),
        HandleMsg::BeforeTrade{} => before_trade(deps, info),
        HandleMsg::SetVault{ vault_address } => set_vault_address(deps, info, vault_address)
    }
}

pub fn before_trade(
    deps: DepsMut,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut conf = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != conf.vault_address {
        return Err(StdError::generic_err("Unauthorized."));
    }


    conf.last_balance = query_balance(&deps.querier, info.sender, conf.denom.clone())?;
    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::default())
}

pub fn after_trade(
    deps: DepsMut,
    info: MessageInfo,
) -> StdResult<Response> {
    let conf = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != conf.vault_address {
        return Err(StdError::generic_err("Unauthorized."));
    }

    let balance = query_balance(&deps.querier, info.sender, conf.denom)?;

    if balance < conf.last_balance {
        return Err(StdError::generic_err("Cancel losing trade."));
    }

    Ok(Response::default())
}

pub fn set_vault_address(
    deps: DepsMut,
    info: MessageInfo,
    vault_address: String
) -> StdResult<Response> {
    let mut conf = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != conf.owner {
        return Err(StdError::generic_err("Unauthorized."));
    }
    conf.vault_address = deps.api.addr_canonicalize(&vault_address)?;
    CONFIG.save(deps.storage, &conf)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps:Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastBalance{} => to_binary(&try_query_last_balance(deps)?),
        QueryMsg::Vault{} => to_binary(&try_query_vault_address(deps)?),
    }
}

pub fn try_query_last_balance(deps: Deps) -> StdResult<LastBalanceResponse> {
    let conf = CONFIG.load(deps.storage)?;
    Ok(LastBalanceResponse{ last_balance: conf.last_balance })
}

pub fn try_query_vault_address(deps: Deps) -> StdResult<VaultResponse> {
    let conf = CONFIG.load(deps.storage)?;
    Ok(VaultResponse{ vault_address: deps.api.addr_humanize(&conf.vault_address)? })
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{from_binary, Coin, Api};
    use cosmwasm_std::testing::{mock_dependencies, mock_env};


    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string()
        };
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse = from_binary(&query(deps.as_ref(), QueryMsg::LastBalance{}).unwrap()).unwrap();
        assert_eq!(res.last_balance, Uint128::zero());

        let res: VaultResponse = from_binary(&query(deps.as_ref(), QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);
    }

    #[test]
    fn test_set_vault() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string()
        };
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res: VaultResponse = from_binary(&query(deps.as_ref(), QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);

        let other_vault = deps.api.addr_validate("test_vault").unwrap();
        let res = execute(deps.as_mut(), env, info, HandleMsg::SetVault{ vault_address: other_vault.to_string()}).unwrap();
        assert_eq!(0, res.messages.len());

        let res: VaultResponse = from_binary(&query(deps.as_ref(), QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, other_vault);
    }

    #[test]
    fn test_failure_of_profit_check() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string()
        };
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let initial_balance = Uint128::from(100u64);
        deps.querier.update_balance(vault_address.clone(), vec![Coin{denom: msg.denom.clone(), amount: initial_balance}]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let vault_info = MessageInfo{sender: vault_address.clone(), funds: vec![]};
        let res = execute(deps.as_mut(), env.clone(), vault_info.clone(), HandleMsg::BeforeTrade{}).unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse = from_binary(&query(deps.as_ref(), QueryMsg::LastBalance{}).unwrap()).unwrap();
        assert_eq!(res.last_balance, initial_balance);

        deps.querier.update_balance(vault_address, vec![Coin{denom: msg.denom, amount: Uint128::from(99u64)}]);

        let res = execute(deps.as_mut(), env, vault_info, HandleMsg::AfterTrade{});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let res: LastBalanceResponse = from_binary(&query(deps.as_ref(), QueryMsg::LastBalance{}).unwrap()).unwrap();
        assert_eq!(res.last_balance, initial_balance);
    }

    #[test]
    fn test_success_of_profit_check() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string()
        };
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let initial_balance = Uint128::from(100u64);
        deps.querier.update_balance(vault_address.clone(), vec![Coin{denom: msg.denom.clone(), amount: initial_balance}]);

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let vault_info = MessageInfo{sender: vault_address.clone(), funds: vec![]};
        let res = execute(deps.as_mut(), env.clone(), vault_info.clone(), HandleMsg::BeforeTrade{}).unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse = from_binary(&query(deps.as_ref(), QueryMsg::LastBalance{}).unwrap()).unwrap();
        assert_eq!(res.last_balance, initial_balance);

        let res = execute(deps.as_mut(), env, vault_info, HandleMsg::AfterTrade{}).unwrap();
        assert_eq!(0, res.messages.len())
    }

    #[test]
    fn test_check_before_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string()
        };
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let res = execute(deps.as_mut(), env.clone(), info, HandleMsg::BeforeTrade{});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let vault_info = MessageInfo{sender: vault_address.clone(), funds: vec![]};
        let _res = execute(deps.as_mut(), env, vault_info, HandleMsg::BeforeTrade{}).unwrap();
    }

    #[test]
    fn test_check_after_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string()
        };
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let res = execute(deps.as_mut(), env.clone(), info, HandleMsg::AfterTrade{});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let vault_info = MessageInfo{sender: vault_address.clone(), funds: vec![]};
        let _res = execute(deps.as_mut(), env, vault_info, HandleMsg::AfterTrade{}).unwrap();
    }

    #[test]
    fn test_only_owner_can_change_vault() {
        let mut deps = mock_dependencies(&[]);
        let vault_address = deps.api.addr_validate("test_vault").unwrap();
        let other_vault_address = deps.api.addr_validate("other_test_vault").unwrap();
        let msg = InitMsg {
            vault_address: vault_address.to_string(),
            denom: "test".to_string()
        };
        let env = mock_env();
        let owner_info = MessageInfo{sender: deps.api.addr_validate("owner").unwrap(), funds: vec![]};
        let user_info = MessageInfo{sender: deps.api.addr_validate("user").unwrap(), funds: vec![]};

        let _res = instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg.clone()).unwrap();

        let res = execute(deps.as_mut(), env.clone(), user_info, HandleMsg::SetVault{ vault_address: other_vault_address.to_string()});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let res: VaultResponse = from_binary(&query(deps.as_ref(), QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);
    }
}