pub mod contract;
pub mod error;
pub mod pool_info;
pub mod querier;
pub mod response;
pub mod state;

#[cfg(not(target_arch = "wasm32"))]
mod mock;
#[cfg(not(target_arch = "wasm32"))]
pub mod testing {
    pub use crate::mock::mock_dependencies;
}
