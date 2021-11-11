pub const UST_DENOM: &str = "uusd";
pub const LUNA_DENOM: &str = "uluna";

// All denoms are <= 5 char
pub fn is_denom(s: &str) -> bool {
    return if s.len() > 5 { false } else { true };
}
