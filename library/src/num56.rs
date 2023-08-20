use ethnum::{AsI256, AsU256, I256, U256};

pub type U56 = u128;
pub type I56 = i128;

pub struct Constants;

impl Constants {
  fn get_u56_max() -> U56 {
    (1 << 56) - 1
  }

  fn get_i56_max() -> I56 {
    let tmp: u128 = 1 << 55;
    (tmp as I56) - 2
  }
}

////////////////////////////////////////////
pub trait AsU56 {
  fn as_u56(&self) -> U56;
}

impl AsU56 for u128 {
  fn as_u56(&self) -> U56 {
    (self & Constants::get_u56_max()) as U56
  }
}

impl AsU56 for U256 {
  fn as_u56(&self) -> U56 {
    (self & Constants::get_u56_max().as_u256()).as_u128() as U56
  }
}

////////////////////////////////////////////

pub trait AsI56 {
  fn as_i56(&self) -> I56;
}

impl AsI56 for i128 {
  fn as_i56(&self) -> I56 {
    let x = self & Constants::get_u56_max() as i128;
    if (x & (1 << 55)) == 0 {
      x as I56
    } else {
      x | (!Constants::get_u56_max() as I56)
    }
  }
}

impl AsI56 for I256 {
  fn as_i56(&self) -> I56 {
    let x = self & Constants::get_u56_max().as_i256();
    if (x & (1 << 55)) == 0 {
      x.as_i128() as I56
    } else {
      (x | (!Constants::get_u56_max().as_i256())).as_i128() as I56
    }
  }
}

pub trait Num56Trait {
  fn add56(self, other: Self) -> Self;
}

impl Num56Trait for U56 {
  fn add56(self, other: Self) -> Self {
    let sum: u128 = self.as_u56() + other.as_u56();
    assert!(sum <= Constants::get_u56_max(), "U56 overflow when adding");
    sum.as_u56()
  }
}

impl Num56Trait for I56 {
  fn add56(self, other: Self) -> Self {
    let sum: i128 = self.as_i56() + other.as_i56();
    assert!(sum <= Constants::get_i56_max(), "U56 overflow when adding");
    sum.as_i56()
  }
}

// fn printBits256(x: I256) {
//   print!("{} => ", x);
//   for i in 0..=255 {
//     print!("{}", (x >> (255 - i)) & I256::ONE);
//   }
//   println!();
// }
//
// fn printBits128(x: i128) {
//   print!("{} => ", x);
//   for i in 0..=127 {
//     print!("{}", (x >> (127 - i)) & 1);
//   }
//   println!();
// }
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
  fn test_to56bit() {
    assert_eq!(I256::from_str("-23498273401982347190234691872346918723461234").unwrap().as_i56(), I256::from_str("9649415604723598").unwrap().as_i128());
    assert_eq!(I256::from_str("-982734092364982734698273649283746928736496873469").unwrap().as_i56(), I256::from_str("32816923106775043").unwrap().as_i128());
    assert_eq!(I256::from_str("-1082734092364982342734698273649283746928736496873469").unwrap().as_i56(), I256::from_str("-9830521594540029").unwrap().as_i128());
    assert_eq!(I256::from_str("23498273401982347190234691872346918723461234").unwrap().as_i56(), I256::from_str("-9649415604723598").unwrap().as_i128());
    assert_eq!(I256::from_str("982734092364982734698273649283746928736496873469").unwrap().as_i56(), I256::from_str("-32816923106775043").unwrap().as_i128());
    assert_eq!(I256::from_str("1082734092364982342734698273649283746928736496873469").unwrap().as_i56(), I256::from_str("9830521594540029").unwrap().as_i128());

    assert_eq!(U256::from_str("823498273401982347190234691872346918723461234").unwrap().as_u56(), U256::from_str("55793516480503922").unwrap().as_u56());
    assert_eq!(U256::from_str("1082734092364982342734698273649283746928736496873469").unwrap().as_u56(), U256::from_str("9830521594540029").unwrap().as_u56());
    assert_eq!(U256::from_str("982734092364982734698273649283746928736496873469").unwrap().as_u56(), U256::from_str("39240670931152893").unwrap().as_u56());
    assert_eq!(U256::from_str("1239876193874612398476123784969871623498716234987612").unwrap().as_u56(), U256::from_str("15640935760476252").unwrap().as_u56());
    assert_eq!(U256::from_str("6774691723649127834918235491823549126345912378401234501").unwrap().as_u56(), U256::from_str("38319641664953925").unwrap().as_u56());
    assert_eq!(U256::from_str("54789234659871623407126349875123497180162340162340098162").unwrap().as_u56(), U256::from_str("46021751246655602").unwrap().as_u56());
  }
}
