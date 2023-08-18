// use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// use near_sdk::env;
// use near_sdk::ext_contract;
// use near_sdk::serde::{Serialize, Deserialize};
use ethnum::U256;

pub trait BitMathTrait {
  fn most_significant_bit(_x: U256) -> u8;
  fn least_significant_bit(_x: U256) -> u8;
}

pub struct BitMath {}

impl BitMathTrait for BitMath {
  /// @notice Returns the index of the most significant bit of the number,
  ///     where the least significant bit is at index 0 and the most significant bit is at index 255
  /// @dev The function satisfies the property:
  ///     x >= 2**most_significant_bit(x) and x < 2**(most_significant_bit(x)+1)
  /// @param x the value for which to compute the most significant bit, must be greater than 0
  /// @return r the index of the most significant bit
  fn most_significant_bit(_x: U256) -> u8 {
    let mut x = _x;
    assert!(x > 0, "Value must be greater than 0");

    let mut r: u8 = 0;
    if x >= U256::from_str_hex("0x100000000000000000000000000000000").unwrap() {
      x >>= 128;
      r += 128;
    }
    if x >= 0x10000000000000000 {
      x >>= 64;
      r += 64;
    }
    if x >= 0x100000000 {
      x >>= 32;
      r += 32;
    }
    if x >= 0x10000 {
      x >>= 16;
      r += 16;
    }
    if x >= 0x100 {
      x >>= 8;
      r += 8;
    }
    if x >= 0x10 {
      x >>= 4;
      r += 4;
    }
    if x >= 0x4 {
      x >>= 2;
      r += 2;
    }
    if x >= 0x2 {
      r += 1;
    }
    r
  }

  /// @notice Returns the index of the least significant bit of the number,
  ///     where the least significant bit is at index 0 and the most significant bit is at index 255
  /// @dev The function satisfies the property:
  ///     (x & 2**least_significant_bit(x)) != 0 and (x & (2**(least_significant_bit(x)) - 1)) == 0)
  /// @param x the value for which to compute the least significant bit, must be greater than 0
  /// @return r the index of the least significant bit
  fn least_significant_bit(_x: U256) -> u8 {
    let mut x = _x;
    assert!(x > 0, "Value must be greater than 0");

    let mut r: u8 = 255;
    if (x & (U256::MAX >> 128)) > U256::new(0) {
      r -= 128;
    } else {
      x >>= 128;
    }
    if x & U256::MAX >> 64 > U256::new(0) {
      r -= 64;
    } else {
      x >>= 64;
    }
    if x & U256::MAX >> 32 > U256::new(0) {
      r -= 32;
    } else {
      x >>= 32;
    }
    if x & U256::MAX >> 16 > U256::new(0) {
      r -= 16;
    } else {
      x >>= 16;
    }
    if x & U256::MAX >> 8 > U256::new(0) {
      r -= 8;
    } else {
      x >>= 8;
    }
    if x & 0xf > U256::new(0) {
      r -= 4;
    } else {
      x >>= 4;
    }
    if x & 0x3 > U256::new(0) {
      r -= 2;
    } else {
      x >>= 2;
    }
    if x & 0x1 > U256::new(0) {
      r -= 1;
    }
    r
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // use near_sdk::MockedBlockchain;
  use near_sdk::test_utils::VMContextBuilder;
  use near_sdk::testing_env;
  use super::BitMathTrait;
  use super::BitMath;
  use std::str::FromStr;

  #[test]
  fn test_bit_math() {
    let context = VMContextBuilder::new()
      .build();
    testing_env!(context);

    let values = [
      U256::new(1234567890u128),
      U256::from_str("57896044618658097711785492504343953926634992332820282019728792004938939802342").unwrap(),
      U256::from_str("1606938044259977626524336601268507632356953233627996783863721").unwrap(),
      U256::from_str("1037042214541001286141").unwrap(),
      !U256::ZERO,
      U256::from_str("97896044618658097711785492504343953926634992332820282019728792004938939802342").unwrap(),
    ];

    let n = values.len();
    for i in 0..n {
      let x = values[i];
      let mut lsb = 0;
      let mut msb = 0;
      for j in 0..=255 {
        if ((x >> j) & U256::ONE) == U256::ONE {
          lsb = j;
          break;
        }
      }
      for j in 0..=255 {
        if ((x >> j) & U256::ONE) == U256::ONE {
          msb = j;
        }
      }
      println!("{} {} {}", x, lsb, msb);
      assert_eq!(lsb, BitMath::least_significant_bit(x));
      assert_eq!(msb, BitMath::most_significant_bit(x));
    }
  }
}
