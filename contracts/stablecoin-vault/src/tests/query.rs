use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, to_binary, DepsMut, MessageInfo, ReplyOn, SubMsg, WasmMsg};
use cosmwasm_std::{Api, Decimal, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::state::{State, STATE};
use cw20::MinterResponse;

use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use white_whale::fee::*;
use white_whale::ust_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::ust_vault::msg::*;

use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR, };

use crate::tests::mock_querier::mock_dependencies;
use crate::tests::instantiate::mock_instantiate;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw};
