use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, StdError};

use white_whale::memory::item::Memory;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::msg::BaseExecuteMsg;
use white_whale::treasury::dapp_base::state::{BaseState, ADMIN};
use crate::dapp_base::common::MEMORY_CONTRACT;

use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use crate::tests::instantiate::mock_instantiate;

