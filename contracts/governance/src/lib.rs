pub use crate::error::ContractError;

pub mod contract;
mod error;
mod staking;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;
mod validators;
