use cosmwasm_std::Api;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

use white_whale::treasury::dapp_base::state::{BaseState, STATE};
use white_whale_testing::dapp_base::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT, MEMORY_CONTRACT};

use crate::contract::instantiate;
use crate::tests::base_mocks::mocks::instantiate_msg;

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        STATE.load(&deps.storage).unwrap(),
        BaseState {
            memory_addr: deps.api.addr_validate(&MEMORY_CONTRACT).unwrap(),
            treasury_address: deps.api.addr_validate(&TREASURY_CONTRACT).unwrap(),
            trader: deps.api.addr_validate(&TRADER_CONTRACT).unwrap(),
        }
    );
}
