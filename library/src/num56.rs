use std::ops::Shl;
use ethnum::{I256, U256};

pub type U56 = u128;
pub type I56 = i128;

pub struct Constants;

impl Constants {
  fn get_u56_max() -> U56 {
    (1 << 56) - 1
  }

  fn get_i56_max() -> I56 {
    let tmp: u128 = 1 << 23;
    (tmp as I56) - 2
  }
}

use crate::full_math::MathOps;

pub trait Num56Trait {
  fn add56(self, other: Self) -> Self;
}

impl Num56Trait for U56 {
  fn add56(self, other: Self) -> Self {
    let sum: u128 = self.to56bit() + other.to56bit();
    assert!(sum <= Constants::get_u56_max(), "U56 overflow when adding");
    sum
  }
}

impl Num56Trait for I56 {
  fn add56(self, other: Self) -> Self {
    let sum: i128 = self.to56bit() + other.to56bit();
    assert!(sum <= Constants::get_i56_max(), "U56 overflow when adding");
    sum
  }
}

pub trait To56 {
  fn to56bit(self) -> Self;
}

impl To56 for U56 {
  fn to56bit(self) -> U56 {
    self & ((1 << 56) - 1)
  }
}

impl To56 for I56 {
  fn to56bit(self) -> I56 {
    self & ((1 << 56) - 1)
  }
}

impl To56 for I256 {
  fn to56bit(self) -> I256 {
    self & ((1 << 56) - 1)
  }
}

impl To56 for U256 {
  fn to56bit(self) -> U256 {
    self & ((1 << 56) - 1)
  }
}