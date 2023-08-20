use crate::num256::U256;

pub const RESOLUTION: u8 = 96;
pub fn get_q96() -> U256 {
    U256::from(2).pow(U256::from(96))
}
