use cosmwasm_std::Api;
use cosmwasm_std::DepsMut;
use cosmwasm_std::testing::{mock_env, mock_info};
use terraswap::asset::AssetInfo;

use white_whale::treasury::dapp_base::msg::BaseInstantiateMsg;
use white_whale::treasury::dapp_base::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};

use crate::contract::{execute, instantiate};
use crate::msg::*;
use crate::tests::mock_querier::mock_dependencies;

pub(crate) fn instantiate_msg() -> BaseInstantiateMsg {
    BaseInstantiateMsg {
        treasury_address: TREASURY_CONTRACT.to_string(),
        trader: TRADER_CONTRACT.to_string(),
    }
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, instantiate_msg())
        .expect("contract successfully handles InstantiateMsg");
}
