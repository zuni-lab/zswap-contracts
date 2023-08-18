use std::ops::Shl;
use ethnum::{I256, U256};

pub type U160 = U256;
pub type I160 = I256;

pub struct Constants;

impl Constants {
  fn get_u160_max() -> U160 {
    (U160::ONE << 160) - U160::ONE
  }

  fn get_i160_max() -> I160 {
    let tmp:U256 = U256::ONE << 159;
    (tmp.as_i256() as I160) - I160::new(2)
  }
}

use crate::full_math::MathOps;

pub trait Num160Trait {
  fn add160(self, other: Self) -> Self;
}

impl Num160Trait for U160 {
  fn add160(self, other: Self) -> Self {
    let sum: U256 = self + other;
    assert!(sum <= Constants::get_u160_max(), "U160 overflow when adding");
    sum
  }
}

impl Num160Trait for I160 {
  fn add160(self, other: Self) -> Self {
    let sum: I256 = self + other;
    assert!(sum <= Constants::get_i160_max(), "U160 overflow when adding");
    sum
  }
}

pub trait To160 {
  fn to160bit(self) -> Self;
}

impl To160 for U160 {
  fn to160bit(self) -> U160 {
    (self & ((Self::ONE << 160) - Self::ONE))
  }
}

impl To160 for I160 {
  fn to160bit(self) -> I160 {
    (self & ((Self::ONE << 160) - Self::ONE))
  }
}
//
// impl MathOps for U160 {
//   fn gt(self, other: Self) -> Self {
//     if self > other { U160::ONE } else { U256::ZERO }
//   }
//   fn lt(self, other: Self) -> Self {
//     if self < other { U160::ONE } else { U256::ZERO }
//   }
//   fn sub(self, other: Self) -> Self {
//     self.overflowing_sub(other).0.to160bit()
//   }
//   fn add(self, other: Self) -> Self {
//     let sum: U256 = self + other;
//     assert!(sum <= U160MAX, "U160 overflow when adding");
//     sum
//   }
//   fn div(self, other: Self) -> Self {
//     return self / other;
//   }
//   fn modulo(self, other: Self) -> Self {
//     return self % other;
//   }
//   fn mul(self, other: Self) -> Self {
//     assert!(self <= U160MAX.div(other), "U160 overflow when multiplying");
//     self * other
//   }
//   // binary multiplication
//   fn mulmod(self, other: Self, modulo: Self) -> Self {
//     let mut a = self;
//     let mut b = other;
//     a %= modulo;
//     b %= modulo;
//     let mut result: U160 = U160::new(0);
//     while b > 0 {
//       if (b & U160::ONE) == U160::ONE {
//         result = MathOps::addmod(result, a, modulo);
//       }
//       a = MathOps::addmod(a, a, modulo);
//       b = b >> 1;
//     }
//
//     return result;
//   }
//   fn muldiv(self, other: Self, modulo: Self) -> Self {
//     return self * other / modulo;
//   }
//   fn addmod(self, other: Self, modulo: Self) -> Self {
//     let a = self.modulo(modulo);
//     let b = other.modulo(modulo);
//     let remaining_a = modulo.sub(a);
//     let res = if remaining_a <= b {
//       b.sub(remaining_a)
//     } else {
//       a.add(b)
//     };
//     res
//   }
// }
