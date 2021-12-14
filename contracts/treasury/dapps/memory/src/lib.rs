pub mod contract;
pub mod error;
pub mod msg;
pub mod queries;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests;
