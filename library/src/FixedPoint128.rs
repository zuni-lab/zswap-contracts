use ethnum::U256;

pub struct FixedPoint128;

impl FixedPoint128 {
  pub const RESOLUTION: u8 = 128;
  pub fn Q128() -> U256 {
    U256::ONE << U256::new(128)
  }
}
//
// fn main() {
//   // Accessing the constants
//   let resolution = FixedPoint128::RESOLUTION;
//   let q128 = FixedPoint128::Q128;
//
//   println!("Resolution: {}", resolution);
//   println!("Q128: {}", q128);
// }
