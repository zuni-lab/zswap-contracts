use std::ops::Div;
use ethnum::{U256, I256};
use near_sdk::serde::__private::de::Content::U16;
use crate::full_math::FullMathTrait;
use crate::num160::To160;
use super::num160::{U160, I160};

pub struct LiquidityMath;

use super::fixed_point_96::FixedPoint96;
use super::full_math::{MathOps, FullMath};

pub trait LiquidityMathTrait {
  fn add_delta(x: u128, y: i128) -> u128;
}

impl LiquidityMathTrait for LiquidityMath {
  /// @notice Add a signed liquidity delta to liquidity and revert if it overflows or underflows
  /// @param x The liquidity before change
  /// @param y The delta by which liquidity should be changed
  /// @return z The liquidity delta
  fn add_delta(x: u128, y: i128) -> u128 {
    let mut z = 0;
    if y < 0 {
      z = x - ((0 - y) as u128);
      assert!(z < x);
    } else {
      z = x + (y as u128);
      assert!(z >= x);
    }
    z
  }
}

#[cfg(test)]
mod tests {
  use std::panic;
  use super::LiquidityMath;
  use super::LiquidityMathTrait;
  use ethnum::U256;

  #[test]
  fn test_add_delta() {
    assert_eq!(LiquidityMath::add_delta(1, 0), 1);
    assert_eq!(LiquidityMath::add_delta(1, -1), 0);
    assert_eq!(LiquidityMath::add_delta(1, 1), 2);
    // 2**128-15 + 15 overflows
    assert!(panic::catch_unwind(|| {
      LiquidityMath::add_delta((U256::new(2).pow(128) - U256::new(15)).as_u128(), 15);
    }).is_err());
    // 0 + -1 underflows
    assert!(panic::catch_unwind(|| {
      LiquidityMath::add_delta(0, -1);
    }).is_err());
    // 3 + -4 underflows
    assert!(panic::catch_unwind(|| {
      LiquidityMath::add_delta(3, -4);
    }).is_err());
  }
}