use ethnum::{AsI256, AsU256, I256, U256};

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

////////////////////////////////////////////
pub trait AsU24 {
  fn as_u24(self) -> U24;
}

impl AsU24 for U24 {
  fn as_u24(self) -> U24 {
    self & Constants::get_u24_max()
  }
}

impl AsU24 for u128 {
  fn as_u24(self) -> U24 {
    (self & Constants::get_u24_max() as u128) as U24
  }
}

impl AsU24 for U256 {
  fn as_u24(self) -> U24 {
    (self & Constants::get_u24_max().as_u256()).as_u32() as U24
  }
}

////////////////////////////////////////////

pub trait AsI24 {
  fn as_i24(self) -> I24;
}

impl AsI24 for I24 {
  fn as_i24(self) -> I24 {
    let x = self & Constants::get_u24_max() as I24;
    if (x & (1 << 23)) == 0 {
      x as I24
    } else {
      x | (!Constants::get_u24_max() as I24)
    }
  }
}

impl AsI24 for i128 {
  fn as_i24(self) -> I24 {
    let x = (self & Constants::get_u24_max() as i128) as i32;
    if (x & (1 << 23)) == 0 {
      x as I24
    } else {
      x | (!Constants::get_u24_max() as I24)
    }
  }
}

impl AsI24 for I256 {
  fn as_i24(self) -> I24 {
    let x = self & Constants::get_u24_max().as_i256();
    if (x & (1 << 23)) == 0 {
      x.as_i32() as I24
    } else {
      (x | (!Constants::get_u24_max().as_i256())).as_i32() as I24
    }
  }
}

pub trait Num24Trait {
  fn add24(self, other: Self) -> Self;
}

impl Num24Trait for U24 {
  fn add24(self, other: Self) -> Self {
    let sum: u32 = self.as_u24() + other.as_u24();
    assert!(sum <= Constants::get_u24_max(), "U24 overflow when adding");
    sum.as_u24()
  }
}

impl Num24Trait for I24 {
  fn add24(self, other: Self) -> Self {
    let sum: i32 = self.as_i24() + other.as_i24();
    assert!(sum <= Constants::get_i24_max(), "U24 overflow when adding");
    sum.as_i24()
  }
}
//
// fn printBits256(x: I256) {
//   print!("{} => ", x);
//   for i in 0..=255 {
//     print!("{}", (x >> (255 - i)) & I256::ONE);
//   }
//   println!();
// }
//
// fn printBits32(x: i32) {
//   print!("{} => ", x);
//   for i in 0..=31 {
//     print!("{}", (x >> (31 - i)) & 1);
//   }
//   println!();
// }

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn test_to24bit() {
    assert_eq!(I256::from_str("-23498273401982347190234691872346918723461234").unwrap().as_i24(), I256::from_str("6872974").unwrap().as_i32());
    assert_eq!(I256::from_str("-982734092364982734698273649283746928736496873469").unwrap().as_i24(), I256::from_str("6721539").unwrap().as_i32());
    assert_eq!(I256::from_str("-1082734092364982342734698273649283746928736496873469").unwrap().as_i24(), I256::from_str("6721539").unwrap().as_i32());
    assert_eq!(I256::from_str("23498273401982347190234691872346918723461234").unwrap().as_i24(), I256::from_str("-6872974").unwrap().as_i32());
    assert_eq!(I256::from_str("982734092364982734698273649283746928736496873469").unwrap().as_i24(), I256::from_str("-6721539").unwrap().as_i32());
    assert_eq!(I256::from_str("1082734092364982342734698273649283746928736496873469").unwrap().as_i24(), I256::from_str("-6721539").unwrap().as_i32());

    assert_eq!(U256::from_str("823498273401982347190234691872346918723461234").unwrap().as_u24(), U256::from_str("9904242").unwrap().as_u24());
    assert_eq!(U256::from_str("1082734092364982342734698273649283746928736496873469").unwrap().as_u24(), U256::from_str("10055677").unwrap().as_u24());
    assert_eq!(U256::from_str("982734092364982734698273649283746928736496873469").unwrap().as_u24(), U256::from_str("10055677").unwrap().as_u24());
    assert_eq!(U256::from_str("1239876193874612398476123784969871623498716234987612").unwrap().as_u24(), U256::from_str("16070748").unwrap().as_u24());
    assert_eq!(U256::from_str("6774691723649127834918235491823549126345912378401234501").unwrap().as_u24(), U256::from_str("6037061").unwrap().as_u24());
    assert_eq!(U256::from_str("54789234659871623407126349875123497180162340162340098162").unwrap().as_u24(), U256::from_str("16748658").unwrap().as_u24());
  }
}
