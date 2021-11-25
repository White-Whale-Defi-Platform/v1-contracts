pub mod contract;
pub mod error;
pub mod msg;
pub mod state;
pub mod astroport_msg;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests;
