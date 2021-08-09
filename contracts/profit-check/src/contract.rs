use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError,
    StdResult, Storage, Uint128
};
use terraswap::querier::{query_balance};

use white_whale::profit_check::msg::{HandleMsg, InitMsg, QueryMsg, LastBalanceResponse, VaultResponse};
use crate::state::{config, config_read, State};


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
        vault_address: deps.api.canonical_address(&msg.vault_address)?,
        denom: msg.denom,
        last_balance: Uint128(0)
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<CosmosMsg>> {
    match msg {
        HandleMsg::AfterTrade{} => after_trade(deps, env),
        HandleMsg::BeforeTrade{} => before_trade(deps, env),
        HandleMsg::SetVault{ vault_address } => set_vault_address(deps, env, vault_address)
    }
}

pub fn before_trade<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse<CosmosMsg>> {
    let mut conf = config(&mut deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != conf.vault_address {
        return Err(StdError::unauthorized());
    }

    conf.last_balance = query_balance(deps, &env.message.sender, conf.denom.clone())?;
    config(&mut deps.storage).save(&conf)?;
 
    Ok(HandleResponse::default())
}

pub fn after_trade<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse<CosmosMsg>> {
    let conf = config_read(&deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != conf.vault_address {
        return Err(StdError::unauthorized());
    }

    let balance = query_balance(deps, &env.message.sender, conf.denom)?;

    if balance < conf.last_balance {
        return Err(StdError::generic_err("Cancel losing trade."));
    }
 
    Ok(HandleResponse::default())
}

pub fn set_vault_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    vault_address: HumanAddr
) -> StdResult<HandleResponse<CosmosMsg>> {
    let mut conf = config_read(&deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != conf.owner {
        return Err(StdError::unauthorized());
    }
    conf.vault_address = deps.api.canonical_address(&vault_address)?;
    config(&mut deps.storage).save(&conf)?;
 
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastBalance{} => to_binary(&try_query_last_balance(deps)?),
        QueryMsg::Vault{} => to_binary(&try_query_vault_address(deps)?),
    }
}

pub fn try_query_last_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<LastBalanceResponse> {
    let conf = config_read(&deps.storage).load()?;
    Ok(LastBalanceResponse{ last_balance: conf.last_balance })
}

pub fn try_query_vault_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<VaultResponse> {
    let conf = config_read(&deps.storage).load()?;
    Ok(VaultResponse{ vault_address: deps.api.human_address(&conf.vault_address)? })
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{from_binary, Coin, coins, HumanAddr};
    use cosmwasm_std::testing::{mock_dependencies, mock_env};


    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        let vault_address = HumanAddr::from("test_vault");
        let msg = InitMsg {
            vault_address: vault_address.clone(),
            denom: "test".to_string()
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse = from_binary(&query(&deps, QueryMsg::LastBalance{}).unwrap()).unwrap();
        assert_eq!(res.last_balance, Uint128(0));

        let res: VaultResponse = from_binary(&query(&deps, QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);
    }

    #[test]
    fn test_set_vault() {
        let mut deps = mock_dependencies(20, &[]);
        let vault_address = HumanAddr::from("test_vault");
        let msg = InitMsg {
            vault_address: vault_address.clone(),
            denom: "test".to_string()
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        let res = init(&mut deps, env.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res: VaultResponse = from_binary(&query(&deps, QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);
        
        let res = handle(&mut deps, env, HandleMsg::SetVault{ vault_address: HumanAddr::from("other")}).unwrap();
        assert_eq!(0, res.messages.len());

        let res: VaultResponse = from_binary(&query(&deps, QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, HumanAddr::from("other"));
    }

    #[test]
    fn test_failure_of_profit_check() {
        let mut deps = mock_dependencies(20, &[]);
        let vault_address = HumanAddr::from("test_vault");
        let denom = "test".to_string();
        let initial_balance = Uint128(100);
        deps.querier.update_balance(vault_address.clone(), vec![Coin{denom: denom.clone(), amount: initial_balance}]);

        let msg = InitMsg {
            vault_address: vault_address.clone(),
            denom: denom.clone()
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        let res = init(&mut deps, env.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let vault_env = mock_env(vault_address.clone(), &coins(1000, "earth"));
        let res = handle(&mut deps, vault_env.clone(), HandleMsg::BeforeTrade{}).unwrap();
        assert_eq!(0, res.messages.len());

        let res: LastBalanceResponse = from_binary(&query(&deps, QueryMsg::LastBalance{}).unwrap()).unwrap();
        assert_eq!(res.last_balance, initial_balance);

        deps.querier.update_balance(vault_address, vec![Coin{denom, amount: Uint128(99)}]);

        let res = handle(&mut deps, vault_env, HandleMsg::AfterTrade{});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let res: LastBalanceResponse = from_binary(&query(&deps, QueryMsg::LastBalance{}).unwrap()).unwrap();
        assert_eq!(res.last_balance, initial_balance);
    }

    #[test]
    fn test_success_of_profit_check() {
        let mut deps = mock_dependencies(20, &[]);
        let vault_address = HumanAddr::from("test_vault");
        let denom = "test".to_string();
        deps.querier.update_balance(vault_address.clone(), vec![Coin{denom: denom.clone(), amount: Uint128(100)}]);

        let msg = InitMsg {
            vault_address: vault_address.clone(),
            denom: denom.clone()
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        let res = init(&mut deps, env.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let vault_env = mock_env(vault_address.clone(), &coins(1000, "earth"));
        let res = handle(&mut deps, vault_env.clone(), HandleMsg::BeforeTrade{}).unwrap();
        assert_eq!(0, res.messages.len());

        deps.querier.update_balance(vault_address, vec![Coin{denom, amount: Uint128(100)}]);

        let res = handle(&mut deps, vault_env, HandleMsg::AfterTrade{}).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_check_before_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(20, &[]);
        let vault_address = HumanAddr::from("test_vault");
        let denom = "test".to_string();
        deps.querier.update_balance(vault_address.clone(), vec![Coin{denom: denom.clone(), amount: Uint128(100)}]);

        let msg = InitMsg {
            vault_address: vault_address.clone(),
            denom: denom.clone()
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        let res = init(&mut deps, env.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = handle(&mut deps, env, HandleMsg::BeforeTrade{});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let vault_env = mock_env(vault_address, &coins(1000, "earth"));
        let _res = handle(&mut deps, vault_env.clone(), HandleMsg::BeforeTrade{}).unwrap();
    }

    #[test]
    fn test_check_after_trade_fails_if_unauthorized() {
        let mut deps = mock_dependencies(20, &[]);
        let vault_address = HumanAddr::from("test_vault");
        let denom = "test".to_string();
        deps.querier.update_balance(vault_address.clone(), vec![Coin{denom: denom.clone(), amount: Uint128(100)}]);

        let msg = InitMsg {
            vault_address: vault_address.clone(),
            denom: denom.clone()
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        let res = init(&mut deps, env.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = handle(&mut deps, env, HandleMsg::AfterTrade{});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let vault_env = mock_env(vault_address, &coins(1000, "earth"));
        let _res = handle(&mut deps, vault_env.clone(), HandleMsg::AfterTrade{}).unwrap();
    }

    #[test]
    fn test_only_owner_can_change_vault() {
        let mut deps = mock_dependencies(20, &[]);
        let vault_address = HumanAddr::from("test_vault");
        let other_vault_address = HumanAddr::from("other_test_vault");
        let denom = "test".to_string();
        deps.querier.update_balance(vault_address.clone(), vec![Coin{denom: denom.clone(), amount: Uint128(100)}]);

        let msg = InitMsg { 
            vault_address: vault_address.clone(),
            denom: denom.clone()
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        let res = init(&mut deps, env.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        
        let vault_env = mock_env(vault_address.clone(), &coins(1000, "earth"));
        let res = handle(&mut deps, vault_env, HandleMsg::SetVault{ vault_address: other_vault_address.clone()});
        match res {
            Err(..) => {},
            _ => panic!("unexpected")
        }

        let res: VaultResponse = from_binary(&query(&deps, QueryMsg::Vault{}).unwrap()).unwrap();
        assert_eq!(res.vault_address, vault_address);
    }
}
