#![allow(unused_doc_comments)]
#![allow(dead_code)]

pub mod bit_math;
pub mod fixed_point_128;
pub mod fixed_point_96;
pub mod sqrt_price_math;
pub mod full_math;
mod liquidity_math;
pub mod math;
mod num160;
mod num24;
mod num56;
mod position;
mod swap_math;
mod tick;
mod tick_bitmap;
pub mod tick_math;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

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
