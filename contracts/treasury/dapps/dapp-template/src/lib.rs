mod commands;
pub mod contract;
pub mod msg;

#[cfg(not(target_arch = "wasm32"))]
pub mod tests;
