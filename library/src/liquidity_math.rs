use ethnum::{U256};

use crate::full_math::FullMathTrait;
use super::num160::{U160};

use super::fixed_point_96;
use super::full_math::{MathOps, FullMath};

/// @notice Add a signed liquidity delta to liquidity and revert if it overflows or underflows
/// @param x The liquidity before change
/// @param y The delta by which liquidity should be changed
/// @return z The liquidity delta
pub(crate) fn add_delta(x: u128, y: i128) -> u128 {
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

/// $L = \frac{\Delta x \sqrt{P_u} \sqrt{P_l}}{\Delta \sqrt{P}}$
fn get_liquidity_for_amount_0(
  _sqrt_price_a_x96: U160,
  _sqrt_price_b_x96: U160,
  amount_0: U256,
) -> u128 {
  let (sqrt_price_a_x96, sqrt_price_b_x96) =
    if _sqrt_price_a_x96 > _sqrt_price_b_x96 {
      (_sqrt_price_b_x96, _sqrt_price_a_x96)
    } else {
      (_sqrt_price_a_x96, _sqrt_price_b_x96)
    };

  let intermediate = FullMath::mul_div(U256::from(sqrt_price_a_x96), U256::from(sqrt_price_b_x96), U256::from(fixed_point_96::get_q96()));
  let liquidity = FullMath::mul_div(U256::from(amount_0), intermediate, sqrt_price_b_x96.sub(sqrt_price_a_x96));
  liquidity.as_u128()
}

/// $L = \frac{\Delta y}{\Delta \sqrt{P}}$
fn get_liquidity_for_amount_1(
  _sqrt_price_a_x96: U160,
  _sqrt_price_b_x96: U160,
  amount_1: U256,
) -> u128 {
  let (sqrt_price_a_x96, sqrt_price_b_x96) =
    if _sqrt_price_a_x96 > _sqrt_price_b_x96 {
      (_sqrt_price_b_x96, _sqrt_price_a_x96)
    } else {
      (_sqrt_price_a_x96, _sqrt_price_b_x96)
    };

  let liquidity = FullMath::mul_div(U256::from(amount_1), U256::from(fixed_point_96::get_q96()), U256::from(sqrt_price_b_x96.sub(sqrt_price_a_x96)));
  liquidity.as_u128()
}

fn get_liquidity_for_amounts(
  sqrt_price_x96: U160,
  _sqrt_price_a_x96: U160,
  _sqrt_price_b_x96: U160,
  amount_0: U256,
  amount_1: U256,
) -> u128 {
  let (sqrt_price_a_x96, sqrt_price_b_x96) =
    if _sqrt_price_a_x96 > _sqrt_price_b_x96 {
      (_sqrt_price_b_x96, _sqrt_price_a_x96)
    } else {
      (_sqrt_price_a_x96, _sqrt_price_b_x96)
    };

  let liquidity: u128;
  if sqrt_price_x96 <= sqrt_price_a_x96 {
    liquidity = get_liquidity_for_amount_0(sqrt_price_a_x96, sqrt_price_b_x96, amount_0);
  } else if sqrt_price_x96 <= sqrt_price_b_x96 {
    let liquidity_0 = get_liquidity_for_amount_0(sqrt_price_x96, sqrt_price_b_x96, amount_0);
    let liquidity_1 = get_liquidity_for_amount_1(sqrt_price_a_x96, sqrt_price_x96, amount_1);
    liquidity = if liquidity_0 < liquidity_1 {
      liquidity_0
    } else {
      liquidity_1
    };
  } else {
    liquidity = get_liquidity_for_amount_1(sqrt_price_a_x96, sqrt_price_b_x96, amount_1);
  }
  liquidity
}

fn add_liquidity(x: u128, y: i128) -> u128 {
  let z: u128 = if y < 0 {
    x - (y as u128)
  } else {
    x + (y as u128)
  };
  z
}

#[cfg(test)]
mod tests {
  use std::panic;
  use ethnum::U256;
  use crate::liquidity_math::add_delta;

  #[test]
  fn test_add_delta() {
    assert_eq!(add_delta(1, 0), 1);
    assert_eq!(add_delta(1, -1), 0);
    assert_eq!(add_delta(1, 1), 2);
    // 2**128-15 + 15 overflows
    assert!(panic::catch_unwind(|| {
      add_delta((U256::new(2).pow(128) - U256::new(15)).as_u128(), 15);
    }).is_err());
    // 0 + -1 underflows
    assert!(panic::catch_unwind(|| {
      add_delta(0, -1);
    }).is_err());
    // 3 + -4 underflows
    assert!(panic::catch_unwind(|| {
      add_delta(3, -4);
    }).is_err());
  }
}