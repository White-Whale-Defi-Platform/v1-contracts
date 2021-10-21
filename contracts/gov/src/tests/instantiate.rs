use cosmwasm_std::{coins, DepsMut};
use cosmwasm_std::{Api, CanonicalAddr, Decimal, Uint128};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_env, mock_info};

use crate::contract::{execute, instantiate};
use crate::mock_querier::mock_dependencies;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, config_read, State, state_read};
use crate::tests::common::{
    DEFAULT_EXPIRATION_PERIOD, DEFAULT_FIX_PERIOD, DEFAULT_PROPOSAL_DEPOSIT, DEFAULT_QUORUM,
    DEFAULT_THRESHOLD, DEFAULT_TIMELOCK_PERIOD, DEFAULT_VOTING_PERIOD, TEST_CREATOR, VOTING_TOKEN,
};

pub(crate) fn instantiate_msg() -> InstantiateMsg {
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

// Mocks instantiation
pub fn mock_instantiate(deps: DepsMut) {
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

/**
 * Tests instantiation of the contract
 */
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
