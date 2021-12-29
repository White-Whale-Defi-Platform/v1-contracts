use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::DepsMut;
use cosmwasm_std::{Api, Decimal};

use white_whale::treasury::dapp_base::msg::BaseInstantiateMsg;
use white_whale::treasury::dapp_base::state::{BaseState, STATE};
use white_whale_testing::dapp_base::common::MEMORY_CONTRACT;

use crate::contract::instantiate;
use crate::msg::InstantiateMsg;
use crate::tests::base_mocks::mocks::instantiate_msg as base_init_msg;
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};

pub(crate) fn vault_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        base: base_init_msg(),
        memory_addr: MEMORY_CONTRACT.into(),
        token_code_id: 3u64,
        fee: Decimal::zero(),
        deposit_asset: TREASURY_CONTRACT.to_string(),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    }
}

// /**
//  * Tests successful instantiation of the contract.
//  */
// #[test]
// fn successful_initialization() {
//     let mut deps = mock_dependencies(&[]);

//     let base_msg = base_init_msg();
//     let
//     let info = mock_info(TEST_CREATOR, &[]);
//     let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
//     assert_eq!(0, res.messages.len());

//     assert_eq!(
//         STATE.load(&deps.storage).unwrap(),
//         BaseState {
//             treasury_address: deps.api.addr_validate(&TREASURY_CONTRACT).unwrap(),
//             trader: deps.api.addr_validate(&TRADER_CONTRACT).unwrap(),
//         }
//     );
// }
