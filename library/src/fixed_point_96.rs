use ethnum::U256;

pub const RESOLUTION: u8 = 96;
pub fn get_q96() -> U256 {
  U256::new(1 << 96)
}
