use cosmwasm_std::Uint128;
use serde::{Deserialize, Serialize};
use terraswap::asset::AssetInfo;

/// ## Description
/// This structure describes the query messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Validates assets and calculates a new average amount with updated precision
    Consult {
        /// the assets to validate
        token: AssetInfo,
        /// the amount
        amount: Uint128,
    },
}
