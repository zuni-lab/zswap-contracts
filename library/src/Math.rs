use std::ops::Div;
use ethnum::U256;

pub struct Math;

use super::FixedPoint96;

impl Math {
  // /// @notice Calculates amount0 delta between two prices
  // pub fn calc_amount0_delta(
  //   sqrt_price_a_x96: u128,
  //   sqrt_price_b_x96: u128,
  //   liquidity: u128,
  //   round_up: bool,
  // ) -> U256 {
  //   let (sqrt_price_a_x96, sqrt_price_b_x96) =
  //     if sqrt_price_a_x96 > sqrt_price_b_x96 {
  //       (sqrt_price_b_x96, sqrt_price_a_x96)
  //     } else {
  //       (sqrt_price_a_x96, sqrt_price_b_x96)
  //     };
  //   // b >= a
  //
  //   assert!(sqrt_price_a_x96 > 0);
  //
  //   let numerator1 = U256::from(liquidity) << FixedPoint96::RESOLUTION;
  //   let numerator2 = U256::from(sqrt_price_b_x96 - sqrt_price_a_x96);
  //
  //   let amount0: U256 = if round_up {
  //     Math::div_rounding_up(
  //       Math::mul_div_rounding_up(numerator1, numerator2, sqrt_price_b_x96),
  //       sqrt_price_a_x96,
  //     )
  //   } else {
  //     PRBU256::mul_div(numerator1, numerator2, sqrt_price_b_x96) / sqrt_price_a_x96
  //   };
  //
  //   amount0
  // }
  //
  // pub fn calc_amount1_delta(
  //   sqrt_price_a_x96: u128,
  //   sqrt_price_b_x96: u128,
  //   liquidity: u128,
  //   round_up: bool,
  // ) -> U256 {
  //   let (sqrt_price_a_x96, sqrt_price_b_x96) =
  //     if sqrt_price_a_x96 > sqrt_price_b_x96 {
  //       (sqrt_price_b_x96, sqrt_price_a_x96)
  //     } else {
  //       (sqrt_price_a_x96, sqrt_price_b_x96)
  //     };
  //
  //   let amount1: U256 = if round_up {
  //     Math::mul_div_rounding_up(liquidity, sqrt_price_b_x96 - sqrt_price_a_x96, FixedPoint96::Q96)
  //   } else {
  //     PRBU256::mul_div(liquidity, sqrt_price_b_x96 - sqrt_price_a_x96, FixedPoint96::Q96)
  //   };
  //
  //   amount1
  // }
  //
  // pub fn calc_amount0_delta_int(
  //   sqrt_price_a_x96: u128,
  //   sqrt_price_b_x96: u128,
  //   liquidity: i128,
  // ) -> i128 {
  //   let amount0 = if liquidity < 0 {
  //     -(Math::calc_amount0_delta(sqrt_price_a_x96, sqrt_price_b_x96, -liquidity as u128, false) as i128)
  //   } else {
  //     Math::calc_amount0_delta(sqrt_price_a_x96, sqrt_price_b_x96, liquidity as u128, true) as i128
  //   };
  //
  //   amount0
  // }
  //
  // pub fn calc_amount1_delta_int(
  //   sqrt_price_a_x96: u128,
  //   sqrt_price_b_x96: u128,
  //   liquidity: i128,
  // ) -> i128 {
  //   let amount1 = if liquidity < 0 {
  //     -(Math::calc_amount1_delta(sqrt_price_a_x96, sqrt_price_b_x96, -liquidity as u128, false) as i128)
  //   } else {
  //     Math::calc_amount1_delta(sqrt_price_a_x96, sqrt_price_b_x96, liquidity as u128, true) as i128
  //   };
  //
  //   amount1
  // }
  //
  // fn get_next_sqrt_price_from_input(
  //   sqrt_price_x96: u160,
  //   liquidity: u128,
  //   amount_in: U256,
  //   zero_for_one: bool,
  // ) -> u160 {
  //   if zero_for_one {
  //     get_next_sqrt_price_from_amount0_rounding_up(sqrt_price_x96, liquidity, amount_in)
  //   } else {
  //     get_next_sqrt_price_from_amount1_rounding_down(sqrt_price_x96, liquidity, amount_in)
  //   }
  // }
  //
  // fn get_next_sqrt_price_from_amount0_rounding_up(
  //   sqrt_price_x96: u160,
  //   liquidity: u128,
  //   amount_in: U256,
  // ) -> u160 {
  //   let numerator = U256::from(liquidity) << FixedPoint96::RESOLUTION;
  //   let product = amount_in * U256::from(sqrt_price_x96);
  //
  //   // If product doesn't overflow, use the precise formula.
  //   if product / amount_in == U256::from(sqrt_price_x96) {
  //     let denominator = numerator + product;
  //     if denominator >= numerator {
  //       return numerator.mul_div_rounding_up(sqrt_price_x96, denominator).into();
  //     }
  //   }
  //
  //   // If product overflows, use a less precise formula.
  //   return numerator.div_rounding_up(numerator.div(sqrt_price_x96) + amount_in).into();
  // }
  //
  // fn get_next_sqrt_price_from_amount1_rounding_down(
  //   sqrt_price_x96: u160,
  //   liquidity: u128,
  //   amount_in: U256,
  // ) -> u160 {
  //   return (U256::from(sqrt_price_x96) + amount_in.mul_div(FixedPoint96::Q96, liquidity)).into();
  // }
  //
  // fn mul_div_rounding_up(a: U256, b: U256, denominator: U256) -> U256 {
  //   let result = a.mul_div(b, denominator);
  //   if a.mulmod(b, denominator) > 0 {
  //     assert!(result < U256::max_value());
  //     result + U256::one()
  //   } else {
  //     result
  //   }
  // }
  //
  // fn div_rounding_up(numerator: U256, denominator: U256) -> U256 {
  //   numerator.div(denominator) + (numerator % denominator > 0)
  // }
}
