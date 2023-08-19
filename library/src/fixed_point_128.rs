use ethnum::U256;
use std::str::FromStr;

pub const RESOLUTION: u8 = 128;

pub fn get_q128() -> U256 {
  U256::from_str("340282366920938463463374607431768211456").unwrap()
}

//
// fn main() {
//   // Accessing the constants
//   let resolution = fixed_point_128::RESOLUTION;
//   let q128 = fixed_point_128::get_q128();
//
//   println!("Resolution: {}", resolution);
//   println!("get_q128: {}", q128);
// }
