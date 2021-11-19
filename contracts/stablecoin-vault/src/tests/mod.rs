// use crate::error::StableVaultError;

mod callback;
mod common;
mod flashloan;
pub mod instantiate;
mod integration_test;
pub mod common_integration;
#[cfg(test)]
mod mock_querier;
mod pool;

mod state;
mod deposit;
mod whitelist;
mod query;
