pub mod bit_math;
pub mod fixed_point_128;
pub mod fixed_point_96;
pub mod full_math;
pub mod liquidity_math;
pub mod num160;
pub mod num24;
pub mod num56;
pub mod position;
pub mod sqrt_price_math;
pub mod swap_math;
pub mod tick;
pub mod tick_bitmap;
pub mod tick_math;

#[cfg(test)]
mod tests {
    // use super::*;

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
