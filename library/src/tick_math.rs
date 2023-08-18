use std::str::FromStr;
use ethnum::{AsI256, AsU256, I256, U256};
use crate::num160::{To160, U160};
use crate::num24::{I24, To24};
// use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// use near_sdk::{env, log};
// use near_sdk::ext_contract;

struct TickConstants;

impl TickConstants {
  pub const MIN_TICK: I24 = -887272;
  pub const MAX_TICK: I24 = -TickConstants::MIN_TICK;
  pub const MIN_SQRT_RATIO: U160 = U256::new(4295128739);

  pub fn max_sqrt_ratio() -> U160 {
    U160::from_str("1461446703485210103287273052203988822378723970342")
      .unwrap_or_else(|_| U160::ZERO)
  }
}

/// @title Math library for computing sqrt prices from ticks and vice versa
/// @notice Computes sqrt price for ticks of size 1.0001, i.e. sqrt(1.0001^tick) as fixed point Q64.96 numbers. Supports
/// prices between 2**-128 and 2**128
pub trait TickMathTrait {
  fn get_sqrt_ratio_at_tick(tick: I24) -> U160;
  fn get_tick_at_sqrt_ratio(sqrt_price_x96: U160) -> I24;
}

pub struct TickMath {}

impl TickMathTrait for TickMath {
  /// @notice Calculates sqrt(1.0001^tick) * 2^96
  /// @dev Throws if |tick| > max tick
  /// @param tick The input tick for the above formula
  /// @return sqrtPriceX96 A Fixed point Q64.96 number representing the sqrt of the ratio of the two assets (token1/token0)
  /// at the given tick
  fn get_sqrt_ratio_at_tick(tick: I24) -> U160 {
    /// second inequality must be < because the price can never reach the price at the max tick
    let abs_tick = if tick < 0 { (I256::ZERO - tick.as_i256()).as_u256() } else { tick.as_i256().as_u256() };
    assert!(abs_tick <= TickConstants::max_sqrt_ratio().as_u256(), "Tick out of range");

    let mut ratio = if (abs_tick & 1) != U256::ZERO {
      U256::from_str_hex("0xfffcb933bd6fad37aa2d162d1a594001").unwrap()
    } else {
      U256::from_str_hex("0x100000000000000000000000000000000").unwrap()
    };

    if (abs_tick & 0x2) != U256::ZERO {
      ratio = (ratio * 0xfff97272373d413259a46990580e213a) >> 128;
    }
    if (abs_tick & 0x4) != U256::ZERO {
      ratio = (ratio * 0xfff2e50f5f656932ef12357cf3c7fdcc) >> 128;
    }
    if (abs_tick & 0x8) != U256::ZERO {
      ratio = (ratio * 0xffe5caca7e10e4e61c3624eaa0941cd0) >> 128;
    }
    if (abs_tick & 0x10) != U256::ZERO {
      ratio = (ratio * 0xffcb9843d60f6159c9db58835c926644) >> 128;
    }
    if (abs_tick & 0x20) != U256::ZERO {
      ratio = (ratio * 0xff973b41fa98c081472e6896dfb254c0) >> 128;
    }
    if (abs_tick & 0x40) != U256::ZERO {
      ratio = (ratio * 0xff2ea16466c96a3843ec78b326b52861) >> 128;
    }
    if (abs_tick & 0x80) != U256::ZERO {
      ratio = (ratio * 0xfe5dee046a99a2a811c461f1969c3053) >> 128;
    }
    if (abs_tick & 0x100) != U256::ZERO {
      ratio = (ratio * 0xfcbe86c7900a88aedcffc83b479aa3a4) >> 128;
    }
    if (abs_tick & 0x200) != U256::ZERO {
      ratio = (ratio * 0xf987a7253ac413176f2b074cf7815e54) >> 128;
    }
    if (abs_tick & 0x400) != U256::ZERO {
      ratio = (ratio * 0xf3392b0822b70005940c7a398e4b70f3) >> 128;
    }
    if (abs_tick & 0x800) != U256::ZERO {
      ratio = (ratio * 0xe7159475a2c29b7443b29c7fa6e889d9) >> 128;
    }
    if (abs_tick & 0x1000) != U256::ZERO {
      ratio = (ratio * 0xd097f3bdfd2022b8845ad8f792aa5825) >> 128;
    }
    if (abs_tick & 0x2000) != U256::ZERO {
      ratio = (ratio * 0xa9f746462d870fdf8a65dc1f90e061e5) >> 128;
    }
    if (abs_tick & 0x4000) != U256::ZERO {
      ratio = (ratio * 0x70d869a156d2a1b890bb3df62baf32f7) >> 128;
    }
    if (abs_tick & 0x8000) != U256::ZERO {
      ratio = (ratio * 0x31be135f97d08fd981231505542fcfa6) >> 128;
    }
    if (abs_tick & 0x10000) != 0 {
      ratio = (ratio * 0x9aa508b5b7a84e1c677de54f3e99bc9) >> 128;
    }
    if (abs_tick & 0x20000) != U256::ZERO {
      ratio = (ratio * 0x5d6af8dedb81196699c329225ee604) >> 128;
    }
    if (abs_tick & 0x40000) != U256::ZERO {
      ratio = (ratio * 0x2216e584f5fa1ea926041bedfe98) >> 128;
    }
    if (abs_tick & 0x80000) != U256::ZERO {
      ratio = (ratio * 0x48a170391f7dc42444e8fa2) >> 128;
    }

    if tick > 0 {
      ratio = U256::MAX / ratio;
    }

    // this divides by 1<<32 rounding up to go from a Q128.128 to a Q128.96.
    // we then downcast because we know the result always fits within 160 bits due to our tick input constraint
    // we round up in the division so get_tick_at_sqrt_ratio of the output price is always consistent
    let shifted_ratio = (((ratio >> 32) + U256::new(if (ratio % (1u128 << 32)) == U256::ZERO { 0 } else { 1 })) as U256).to160bit();
    shifted_ratio
  }

  /// @notice Calculates the greatest tick value such that getRatioAtTick(tick) <= ratio
  /// @dev Throws in case sqrtPriceX96 < MIN_SQRT_RATIO, as MIN_SQRT_RATIO is the lowest value getRatioAtTick may
  /// ever return.
  /// @param sqrtPriceX96 The sqrt ratio for which to compute the tick as a Q64.96
  /// @return tick The greatest tick for which the ratio is less than or equal to the input ratio
  fn get_tick_at_sqrt_ratio(sqrt_price_x96: U160) -> I24 {
    /// second inequality must be < because the price can never reach the price at the max tick

    assert!(
      sqrt_price_x96 >= TickConstants::MIN_SQRT_RATIO && sqrt_price_x96 < TickConstants::max_sqrt_ratio(),
      "Sqrt ratio out of range"
    );

    let ratio: U256 = sqrt_price_x96 << 32;
    let mut r = ratio;
    let mut msb: U256 = U256::new(0);

    // assembly {
    //   let f := shl(7, gt(r, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF))
    //   msb := or(msb, f)
    //   r := shr(f, r)

    let f = U256::new(u128::from(r > 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)) << 7;
    msb |= f;
    r >>= f;

    let f = U256::new(u128::from(r > 0xFFFFFFFFFFFFFFFF)) << 6;
    msb |= f;
    r >>= f;

    let f = U256::new(u128::from(r > 0xFFFFFFFF)) << 5;
    msb |= f;
    r >>= f;

    let f = U256::new(u128::from(r > 0xFFFF)) << 4;
    msb |= f;
    r >>= f;

    let f = U256::new(u128::from(r > 0xFF)) << 3;
    msb |= f;
    r >>= f;

    let f = U256::new(u128::from(r > 0xF)) << 2;
    msb |= f;
    r >>= f;

    let f = U256::new(u128::from(r > 0x3)) << 1;
    msb |= f;
    r >>= f;

    let f = U256::new(u128::from(r > 0x1));
    msb |= f;

    if msb >= 128 {
      r = ratio >> (msb - 127);
    } else {
      r = ratio << (127 - msb);
    }

    let mut log_2: I256 = (msb.as_i256() - 128) << 64;

    for i in 0..14 {
      r = (r * r) >> 127;
      let f = I256::from(((r >> 128) as U256).as_u128());
      log_2 = log_2 | (f << (63 - i));
      if i < 13 { r >>= f };
    }

    let log_sqrt10001: I256 = log_2 * 255738958999603826347141; // 128.128 number
    let tick_low = (((log_sqrt10001 - 3402992956809132418596140100660247210i128) >> 128) as I256).to24bit().as_i32();
    let tick_hi = (((log_sqrt10001 + I256::from_str("291339464771989622907027621153398088495").unwrap()) >> 128) as I256).to24bit().as_i32();

    if tick_low == tick_hi {
      tick_low
    } else if TickMath::get_sqrt_ratio_at_tick(tick_hi) <= sqrt_price_x96 {
      tick_hi
    } else {
      tick_low
    }
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
  use std::ops::Mul;
  use super::*;
  // use near_sdk::MockedBlockchain;
  use near_sdk::test_utils::VMContextBuilder;
  use near_sdk::testing_env;
  use super::TickMath;
  use super::TickMathTrait;
  use crate::num24::{I24, To24};
  use std::panic;
  use crate::full_math::MathOps;

  #[test]
  fn test_get_sqrt_ratio_at_tick() {
    // // let context = VMContextBuilder::new()
    // //   .build();
    // // testing_env!(context);
    //
    // //throws for too low
    // assert!(panic::catch_unwind(|| {
    //   TickMath::get_sqrt_ratio_at_tick(TickConstants::MIN_TICK - 1);
    // }).is_err());
    // // //throws for too high
    // // assert!(panic::catch_unwind(|| {
    // //   TickMath::get_sqrt_ratio_at_tick(TickConstants::MAX_TICK + 1);
    // // }).is_err());
    // //
    // // assert_eq!(TickMath::get_sqrt_ratio_at_tick(TickConstants::MIN_TICK), U160::new(4295128739));
    // // assert_eq!(TickMath::get_sqrt_ratio_at_tick(TickConstants::MIN_TICK + 1), U160::new(4295343490));
    // // assert_eq!(TickMath::get_sqrt_ratio_at_tick(TickConstants::MAX_TICK - 1), U160::from_str("1461373636630004318706518188784493106690254656249").unwrap());
    // // // min tick ratio is less than js implementation
    // // // max tick ratio is greater than js implementation
    // // assert_eq!(TickMath::get_sqrt_ratio_at_tick(TickConstants::MAX_TICK), U160::from_str("1461446703485210103287273052203988822378723970342").unwrap());
    // assert_eq!(TickMath::get_sqrt_ratio_at_tick(TickConstants::MIN_TICK), TickConstants::MIN_SQRT_RATIO);
    // assert_eq!(TickMath::get_sqrt_ratio_at_tick(TickConstants::MAX_TICK), TickConstants::max_sqrt_ratio());
  }

  #[test]
  fn test_get_tick_at_sqrt_ratio() {
    // //throws for too low
    // assert!(panic::catch_unwind(|| {
    //   TickMath::get_tick_at_sqrt_ratio(TickConstants::MIN_SQRT_RATIO - U160::ONE);
    // }).is_err());
    // //throws for too high
    // assert!(panic::catch_unwind(|| {
    //   TickMath::get_tick_at_sqrt_ratio(TickConstants::max_sqrt_ratio());
    // }).is_err());
    //
    // assert_eq!(TickMath::get_tick_at_sqrt_ratio(TickConstants::MIN_SQRT_RATIO), TickConstants::MIN_TICK);
    // assert_eq!(TickMath::get_tick_at_sqrt_ratio(U160::new(4295343490)), TickConstants::MIN_TICK + 1);
    // assert_eq!(TickMath::get_tick_at_sqrt_ratio(U160::from_str("1461373636630004318706518188784493106690254656249").unwrap()), TickConstants::MAX_TICK - 1);
    // assert_eq!(TickMath::get_tick_at_sqrt_ratio(TickConstants::max_sqrt_ratio().sub(U160::ONE)), TickConstants::MAX_TICK - 1);
  }
}
