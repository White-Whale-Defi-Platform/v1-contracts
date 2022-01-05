use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Addr;
use cosmwasm_std::Api;

use white_whale::memory::item::Memory;
use white_whale::treasury::dapp_base::msg::BaseInstantiateMsg;
use white_whale::treasury::dapp_base::state::{BaseState, BASESTATE};

use crate::contract::instantiate;
use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};

pub(crate) fn instantiate_msg() -> BaseInstantiateMsg {
    BaseInstantiateMsg {
        treasury_address: TREASURY_CONTRACT.to_string(),
        trader: TRADER_CONTRACT.to_string(),
        memory_addr: MEMORY_CONTRACT.to_string(),
    }
}

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
        BASESTATE.load(&deps.storage).unwrap(),
        BaseState {
            treasury_address: deps.api.addr_validate(&TREASURY_CONTRACT).unwrap(),
            trader: deps.api.addr_validate(&TRADER_CONTRACT).unwrap(),
            memory: Memory {
                address: Addr::unchecked(&MEMORY_CONTRACT.to_string())
            }
        }
    );
}
