use ethnum::U256;

struct FixedPoint96;

impl FixedPoint96 {
  const RESOLUTION: u8 = 96;
  pub fn Q96() -> U256 {
    U256::ONE << U256::new(96)
  }
}

