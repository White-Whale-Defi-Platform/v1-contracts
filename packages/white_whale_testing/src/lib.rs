#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
pub mod anchor_mock;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
pub mod tswap_mock;

#[cfg(not(target_arch = "wasm32"))]
pub mod dapp_base;
