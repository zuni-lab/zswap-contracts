#![allow(unused_doc_comments)]
#![allow(dead_code)]

pub mod TickMath;
pub mod BitMath;
pub mod FixedPoint128;
pub mod FixedPoint96;
pub mod Path;
pub mod Math;
pub mod FullMath;

pub fn add(left: usize, right: usize) -> usize {
  left + right
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }

  // use crate::lib::Math;
  // use crate::lib::TickMath;
  //
  // #[test]
  // fn test_calc_amount_0_delta() {
  //   let amount0 = Math::calc_amount_0_delta(
  //     TickMath::get_sqrt_ratio_at_tick(85176),
  //     TickMath::get_sqrt_ratio_at_tick(86129),
  //     1517882343751509868544,
  //   );
  //
  //   assert_eq!(amount0, 0.998833192822975409);
  // }
  //
  // #[test]
  // fn test_calc_amount_1_delta() {
  //   let amount1 = Math::calc_amount_1_delta(
  //     TickMath::get_sqrt_ratio_at_tick(84222),
  //     TickMath::get_sqrt_ratio_at_tick(85176),
  //     1517882343751509868544,
  //   );
  //
  //   assert_eq!(amount1, 4999.187247111820044641);
  // }
  //
  // #[test]
  // fn test_calc_amount_0_delta_negative() {
  //   let amount0 = Math::calc_amount_0_delta(
  //     TickMath::get_sqrt_ratio_at_tick(85176),
  //     TickMath::get_sqrt_ratio_at_tick(86129),
  //     -1517882343751509868544,
  //   );
  //
  //   assert_eq!(amount0, -0.998833192822975408);
  // }
  //
  // #[test]
  // fn test_calc_amount_1_delta_negative() {
  //   let amount1 = Math::calc_amount_1_delta(
  //     TickMath::get_sqrt_ratio_at_tick(84222),
  //     TickMath::get_sqrt_ratio_at_tick(85176),
  //     -1517882343751509868544,
  //   );
  //
  //   assert_eq!(amount1, -4999.187247111820044640);
  // }
}
