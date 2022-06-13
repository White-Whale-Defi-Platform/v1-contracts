use cosmwasm_std::{testing::mock_info, MessageInfo};

pub const TEST_CREATOR: &str = "creator";
pub const TEST_OWNER: &str = "owner";
pub const TEST_MEMORY_CONTRACT: &str = "memory";

pub fn mock_creator_info() -> MessageInfo {
    mock_info(TEST_CREATOR, &[])
}
