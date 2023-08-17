use std::str::FromStr;
use ethnum::{I256, U256};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log};
use near_sdk::ext_contract;

pub struct TickConstants;

impl TickConstants {
  pub const MIN_TICK: i32 = -887272;
  pub const MAX_TICK: i32 = -TickConstants::MIN_TICK;
  pub const MIN_SQRT_RATIO: U256 = U256::new(4295128739);

  pub fn max_sqrt_ratio() -> U256 {
    U256::from_str("1461446703485210103287273052203988822378723970342")
      .unwrap_or_else(|_| U256::ZERO)
  }
}

/// @title Math library for computing sqrt prices from ticks and vice versa
/// @notice Computes sqrt price for ticks of size 1.0001, i.e. sqrt(1.0001^tick) as fixed point Q64.96 numbers. Supports
/// prices between 2**-128 and 2**128
pub trait TickMathTrait {
  fn get_sqrt_ratio_at_tick(&self, tick: i32) -> U256;
  fn get_tick_at_sqrt_ratio(&self, sqrt_price_x96: U256) -> i32;
}

pub struct TickMath {}

impl TickMathTrait for TickMath {
  /// @notice Calculates sqrt(1.0001^tick) * 2^96
  /// @dev Throws if |tick| > max tick
  /// @param tick The input tick for the above formula
  /// @return sqrtPriceX96 A Fixed point Q64.96 number representing the sqrt of the ratio of the two assets (token1/token0)
  /// at the given tick
  fn get_sqrt_ratio_at_tick(&self, tick: i32) -> U256 {
    /// second inequality must be < because the price can never reach the price at the max tick
    let abs_tick = U256::new(if tick < 0 { -tick } else { tick } as u128);
    assert!(abs_tick <= TickConstants::max_sqrt_ratio(), "Tick out of range");

    let mut ratio = if abs_tick & 1 != 0 {
      U256::from_str_hex("0xfffcb933bd6fad37aa2d162d1a594001").unwrap()
    } else {
      U256::from_str_hex("0x100000000000000000000000000000000").unwrap()
    };

    if abs_tick & 0x2 != 0 {
      ratio = (ratio * 0xfff97272373d413259a46990580e213a) >> 128;
    }
    if abs_tick & 0x4 != 0 {
      ratio = (ratio * 0xfff2e50f5f656932ef12357cf3c7fdcc) >> 128;
    }
    if abs_tick & 0x8 != 0 {
      ratio = (ratio * 0xffe5caca7e10e4e61c3624eaa0941cd0) >> 128;
    }
    if abs_tick & 0x10 != 0 {
      ratio = (ratio * 0xffcb9843d60f6159c9db58835c926644) >> 128;
    }
    if abs_tick & 0x20 != 0 {
      ratio = (ratio * 0xff973b41fa98c081472e6896dfb254c0) >> 128;
    }
    if abs_tick & 0x40 != 0 {
      ratio = (ratio * 0xff2ea16466c96a3843ec78b326b52861) >> 128;
    }
    if abs_tick & 0x80 != 0 {
      ratio = (ratio * 0xfe5dee046a99a2a811c461f1969c3053) >> 128;
    }
    if abs_tick & 0x100 != 0 {
      ratio = (ratio * 0xfcbe86c7900a88aedcffc83b479aa3a4) >> 128;
    }
    if abs_tick & 0x200 != 0 {
      ratio = (ratio * 0xf987a7253ac413176f2b074cf7815e54) >> 128;
    }
    if abs_tick & 0x400 != 0 {
      ratio = (ratio * 0xf3392b0822b70005940c7a398e4b70f3) >> 128;
    }
    if abs_tick & 0x800 != 0 {
      ratio = (ratio * 0xe7159475a2c29b7443b29c7fa6e889d9) >> 128;
    }
    if abs_tick & 0x1000 != 0 {
      ratio = (ratio * 0xd097f3bdfd2022b8845ad8f792aa5825) >> 128;
    }
    if abs_tick & 0x2000 != 0 {
      ratio = (ratio * 0xa9f746462d870fdf8a65dc1f90e061e5) >> 128;
    }
    if abs_tick & 0x4000 != 0 {
      ratio = (ratio * 0x70d869a156d2a1b890bb3df62baf32f7) >> 128;
    }
    if abs_tick & 0x8000 != 0 {
      ratio = (ratio * 0x31be135f97d08fd981231505542fcfa6) >> 128;
    }
    if abs_tick & 0x10000 != 0 {
      ratio = (ratio * 0x9aa508b5b7a84e1c677de54f3e99bc9) >> 128;
    }
    if abs_tick & 0x20000 != 0 {
      ratio = (ratio * 0x5d6af8dedb81196699c329225ee604) >> 128;
    }
    if abs_tick & 0x40000 != 0 {
      ratio = (ratio * 0x2216e584f5fa1ea926041bedfe98) >> 128;
    }
    if abs_tick & 0x80000 != 0 {
      ratio = (ratio * 0x48a170391f7dc42444e8fa2) >> 128;
    }

    if tick > 0 {
      ratio = U256::MAX / ratio;
    }

    // this divides by 1<<32 rounding up to go from a Q128.128 to a Q128.96.
    // we then downcast because we know the result always fits within 160 bits due to our tick input constraint
    // we round up in the division so get_tick_at_sqrt_ratio of the output price is always consistent
    let shifted_ratio: U256 = (ratio >> 32) + U256::new(if ratio % (1 << 32) == 0 { 0 } else { 1 });
    shifted_ratio
  }

  /// @notice Calculates the greatest tick value such that getRatioAtTick(tick) <= ratio
  /// @dev Throws in case sqrtPriceX96 < MIN_SQRT_RATIO, as MIN_SQRT_RATIO is the lowest value getRatioAtTick may
  /// ever return.
  /// @param sqrtPriceX96 The sqrt ratio for which to compute the tick as a Q64.96
  /// @return tick The greatest tick for which the ratio is less than or equal to the input ratio
  fn get_tick_at_sqrt_ratio(&self, sqrt_price_x96: U256) -> i32 {
    /// second inequality must be < because the price can never reach the price at the max tick
    assert!(
      sqrt_price_x96 >= TickConstants::MIN_SQRT_RATIO && sqrt_price_x96 < TickConstants::max_sqrt_ratio(),
      "Sqrt ratio out of range"
    );

    let ratio: U256 = sqrt_price_x96 << 32;
    let mut r = ratio;
    let mut msb: U256 = U256::new(0);

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

    let mut log_2: I256 = (I256::from_le_bytes(msb.to_le_bytes()) - 128) << 64;

    for i in 0..14 {
      r = (r * r) >> 127;
      let f = I256::from(((r >> 128) as U256).as_u128());
      log_2 |= f << (63 - i);
      if i < 13 { r >>= f };
    }

    let log_sqrt10001: I256 = log_2 * 255738958999603826347141; // 128.128 number
    let tick_low = (((log_sqrt10001 - 3402992956809132418596140100660247210i128) >> 128) as I256).as_i32();
    let tick_hi = (((log_sqrt10001 + I256::from_str("291339464771989622907027621153398088495").unwrap()) >> 128) as I256).as_i32();

    if tick_low == tick_hi {
      tick_low
    } else if self.get_sqrt_ratio_at_tick(tick_hi) <= sqrt_price_x96 {
      tick_hi
    } else {
      tick_low
    }
  }
}

impl Default for TickMath {
  fn default() -> Self {
    Self {}
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
  use super::*;
  use near_sdk::MockedBlockchain;
  use near_sdk::test_utils::VMContextBuilder;
  use near_sdk::testing_env;

  #[test]
  fn test_tick_math() {
    let context = VMContextBuilder::new()
      .build();
    testing_env!(context);

    let tick_math = TickMath::default();
    assert_eq!(tick_math.get_tick_at_sqrt_ratio(U256::new(5602223755577321903022134995689)), 85176);
    assert_eq!(tick_math.get_sqrt_ratio_at_tick(85176), 5602223755577321903022134995689); // 5602277097478614198912276234240
  }
}
