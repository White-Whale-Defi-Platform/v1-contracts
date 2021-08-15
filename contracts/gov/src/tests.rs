use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::state::{
    bank_read, bank_store, config_read, poll_store, poll_voter_read, poll_voter_store, state_read,
    Config, Poll, State, TokenManager, PollResponse,PollExecuteMsg, Cw20HookMsg
};
use crate::msg::{ExecuteMsg, InstantiateMsg};


use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, coins, from_binary, to_binary, Addr, Api, CanonicalAddr, CosmosMsg, Decimal, Deps,
    DepsMut, Env, Response, StdError, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::querier::query_token_balance;


const VOTING_TOKEN: &str = "voting_token";
const TEST_CREATOR: &str = "creator";
const TEST_VOTER: &str = "voter1";
const TEST_VOTER_2: &str = "voter2";
const TEST_VOTER_3: &str = "voter3";
const DEFAULT_QUORUM: u64 = 30u64;
const DEFAULT_THRESHOLD: u64 = 50u64;
const DEFAULT_VOTING_PERIOD: u64 = 10000u64;
const DEFAULT_FIX_PERIOD: u64 = 10u64;
const DEFAULT_TIMELOCK_PERIOD: u64 = 10000u64;
const DEFAULT_EXPIRATION_PERIOD: u64 = 20000u64;
const DEFAULT_PROPOSAL_DEPOSIT: u128 = 10000000000u128;

fn create_poll_msg(
    title: String,
    description: String,
    link: Option<String>,
    execute_msg: Option<Vec<PollExecuteMsg>>,
) -> ExecuteMsg {
    ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_CREATOR.to_string(),
        amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        msg: to_binary(&Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            execute_msgs: execute_msg,
        })
        .unwrap(),
    })
}

// Mocks 
fn mock_instantiate(deps: DepsMut) {
    let msg = InstantiateMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, msg)
        .expect("contract successfully handles InstantiateMsg");
}

fn mock_register_voting_token(deps: DepsMut) {
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::RegisterContracts {
        whale_token: VOTING_TOKEN.to_string(),
    };
    let _res = execute(deps, mock_env(), info, msg)
        .expect("contract successfully handles RegisterContracts");
}

fn mock_env_height(height: u64, time: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(time);
    env
}

fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    }
}

// assertion helpers -- perform an action and expect a response
// helper to confirm the expected create_poll response
fn assert_create_poll_result(
    poll_id: u64,
    end_height: u64,
    creator: &str,
    execute_res: Response,
    deps: Deps,
) {
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", creator),
            attr("poll_id", poll_id.to_string()),
            attr("end_height", end_height.to_string()),
        ]
    );

    //confirm poll count
    let state: State = state_read(deps.storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 1,
            total_share: Uint128::zero(),
            total_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        }
    );
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &coins(2, VOTING_TOKEN));
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        Config {
            whale_token: CanonicalAddr::from(vec![]),
            owner: deps.api.addr_canonicalize(&TEST_CREATOR).unwrap(),
            quorum: Decimal::percent(DEFAULT_QUORUM),
            threshold: Decimal::percent(DEFAULT_THRESHOLD),
            voting_period: DEFAULT_VOTING_PERIOD,
            timelock_period: DEFAULT_TIMELOCK_PERIOD,
            expiration_period: DEFAULT_EXPIRATION_PERIOD,
            proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            snapshot_period: DEFAULT_FIX_PERIOD
        }
    );

    let msg = ExecuteMsg::RegisterContracts {
        whale_token: VOTING_TOKEN.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config.whale_token,
        deps.api.addr_canonicalize(&VOTING_TOKEN).unwrap()
    );

    let state: State = state_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::zero(),
            total_deposit: Uint128::zero(),
        }
    );
}

#[test]
fn add_several_execute_msgs() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let info = mock_info(VOTING_TOKEN, &[]);
    let env = mock_env_height(0, 10000);

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();

    let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(12),
    })
    .unwrap();

    let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(1),
    })
    .unwrap();

    // push two execute msgs to the list
    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz,
        },
        PollExecuteMsg {
            order: 3u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz3,
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz2,
        },
    ];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs.clone()),
    );

    let execute_res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_create_poll_result(
        1,
        env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    
}