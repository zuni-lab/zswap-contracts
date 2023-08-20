use crate::num256::U256;

pub const RESOLUTION: u8 = 128;
pub fn get_q128() -> U256 {
    U256::from(2).pow(U256::from(128))
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
