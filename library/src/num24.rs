use std::ops::Shl;
use ethnum::{I256};

pub type U24 = u32;
pub type I24 = i32;

pub struct Constants;

impl Constants {
  fn get_u24_max() -> U24 {
    (1 << 24) - 1
  }

  fn get_i24_max() -> I24 {
    let tmp: u32 = 1 << 23;
    (tmp as I24) - 2
  }
}

use crate::full_math::MathOps;

pub trait Num24Trait {
  fn add24(self, other: Self) -> Self;
}

impl Num24Trait for U24 {
  fn add24(self, other: Self) -> Self {
    let sum: u32 = self.to24bit() + other.to24bit();
    assert!(sum <= Constants::get_u24_max(), "U24 overflow when adding");
    sum
  }
}

impl Num24Trait for I24 {
  fn add24(self, other: Self) -> Self {
    let sum: i32 = self.to24bit() + other.to24bit();
    assert!(sum <= Constants::get_i24_max(), "U24 overflow when adding");
    sum
  }
}

pub trait To24 {
  fn to24bit(self) -> Self;
}

impl To24 for U24 {
  fn to24bit(self) -> U24 {
    (self & ((1 << 24) - 1))
  }
}

impl To24 for I24 {
  fn to24bit(self) -> I24 {
    (self & ((1 << 24) - 1))
  }
}

impl To24 for I256 {
  fn to24bit(self) -> I256 {
    self & ((1 << 24) - 1)
  }
}