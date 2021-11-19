use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, to_binary, DepsMut, MessageInfo, ReplyOn, SubMsg, WasmMsg};
use cosmwasm_std::{Api, Decimal, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::state::{State, STATE};
use cw20::MinterResponse;
use crate::pool_info::{PoolInfo, PoolInfoRaw};

use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use white_whale::fee::*;
use white_whale::ust_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::ust_vault::msg::*;

use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR, };

use crate::tests::mock_querier::mock_dependencies;
use crate::tests::instantiate::mock_instantiate;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw};


// #[test]
// pub fn test_config_query() {

//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());
//     let env = mock_env();
//     let msg = ExecuteMsg::ProvideLiquidity {
//         asset: Asset {
//             info: AssetInfo::NativeToken{
//                 denom: String::from("uusd")
//             },
//             amount: Uint128::new(1000)
//         }
//         };


//     let info = mock_info(TEST_CREATOR, &coins(1000, "uusd"));
//     let execute_res = execute(deps.as_mut(), env.clone(), info, msg);
    

//     let q_res: PoolInfo =
//         from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
//     assert_eq!(
//         q_res.stable_cap,
//         Uint128::from(100_000_000u64)
//     )
// }

#[test]
pub fn test_state_query() {

    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::ProvideLiquidity {
        asset: Asset {
            info: AssetInfo::NativeToken{
                denom: String::from("uusd")
            },
            amount: Uint128::new(1000)
        }
        };


    let info = mock_info(TEST_CREATOR, &coins(1000, "uusd"));
    let execute_res = execute(deps.as_mut(), env.clone(), info, msg);
    

    let q_res: StateResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::State {}).unwrap()).unwrap();
    assert_eq!(
        q_res.allow_non_whitelisted,
        false
    )
}