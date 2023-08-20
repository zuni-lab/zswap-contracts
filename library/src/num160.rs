use ethnum::{I256, U256};

pub type U160 = U256;
pub type I160 = I256;

pub struct Constants;

impl Constants {
    fn get_u160_max() -> U160 {
        (U160::ONE << 160) - U160::ONE
    }

    fn get_i160_max() -> I160 {
        (I160::ONE << I160::new(59)) - I160::new(2)
    }
}

////////////////////////////////////////////
pub trait AsU160 {
    fn as_u160(&self) -> U160;
}

impl AsU160 for U256 {
    fn as_u160(&self) -> U160 {
        self & Constants::get_u160_max()
    }
}

////////////////////////////////////////////

pub trait AsI160 {
    fn as_i160(&self) -> I160;
}

impl AsI160 for I256 {
    fn as_i160(&self) -> I160 {
        let x = self & Constants::get_u160_max().as_i256();
        if (x & (I256::ONE << 159)) == I256::ZERO {
            x as I160
        } else {
            (x | (!Constants::get_u160_max().as_i256())) as I160
        }
    }
}

pub trait Num160Trait {
    fn add160(&self, other: Self) -> Self;
}

impl Num160Trait for U160 {
    fn add160(&self, other: Self) -> Self {
        let sum = self.as_u160() + other.as_u160();
        assert!(
            sum <= Constants::get_u160_max(),
            "U160 overflow when adding"
        );
        sum.as_u160()
    }
}

impl Num160Trait for I160 {
    fn add160(&self, other: Self) -> Self {
        let sum = self.as_i160() + other.as_i160();
        assert!(
            sum <= Constants::get_i160_max(),
            "U160 overflow when adding"
        );
        sum.as_i160()
    }
}

// fn printBits2160(x: I2160) {
//   print!("{} => ", x);
//   for i in 0..=255 {
//     print!("{}", (x >> (255 - i)) & I2160::ONE);
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
    fn test_to160bit() {
        assert_eq!(
            I256::from_str("-23498273401982347190234691872346918723461234")
                .unwrap()
                .as_i160(),
            I256::from_str("-23498273401982347190234691872346918723461234").unwrap()
        );
        assert_eq!(
            I256::from_str("-982734092364982734698273649283746928736496873469")
                .unwrap()
                .as_i160(),
            I256::from_str("478767544965920183505411183432536090919435669507").unwrap()
        );
        assert_eq!(
            I256::from_str("-1082734092364982342734698273649283746928736496873469")
                .unwrap()
                .as_i160(),
            I256::from_str("238620897216719654232187393481970636309517471747").unwrap()
        );
        assert_eq!(
            I256::from_str("23498273401982347190234691872346918723461234")
                .unwrap()
                .as_i160(),
            I256::from_str("23498273401982347190234691872346918723461234").unwrap()
        );
        assert_eq!(
            I256::from_str("982734092364982734698273649283746928736496873469")
                .unwrap()
                .as_i160(),
            I256::from_str("-478767544965920183505411183432536090919435669507").unwrap()
        );
        assert_eq!(
            I256::from_str("1082734092364982342734698273649283746928736496873469")
                .unwrap()
                .as_i160(),
            I256::from_str("-238620897216719654232187393481970636309517471747").unwrap()
        );

        assert_eq!(
            U256::from_str("823498273401982347190234691872346918723461234")
                .unwrap()
                .as_u160(),
            U256::from_str("823498273401982347190234691872346918723461234")
                .unwrap()
                .as_u160()
        );
        assert_eq!(
            U256::from_str("1082734092364982342734698273649283746928736496873469")
                .unwrap()
                .as_u160(),
            U256::from_str("1222880740114183263971497439234312383346415071229")
                .unwrap()
                .as_u160()
        );
        assert_eq!(
            U256::from_str("982734092364982734698273649283746928736496873469")
                .unwrap()
                .as_u160(),
            U256::from_str("982734092364982734698273649283746928736496873469")
                .unwrap()
                .as_u160()
        );
        assert_eq!(
            U256::from_str("1239876193874612398476123784969871623498716234987612")
                .unwrap()
                .as_u160(),
            U256::from_str("522805418006723839399046826463622830485438543964")
                .unwrap()
                .as_u160()
        );
        assert_eq!(
            U256::from_str("6774691723649127834918235491823549126345912378401234501")
                .unwrap()
                .as_u160(),
            U256::from_str("265913065858983492300335843895976173678848908869")
                .unwrap()
                .as_u160()
        );
        assert_eq!(
            U256::from_str("54789234659871623407126349875123497180162340162340098162")
                .unwrap()
                .as_u160(),
            U256::from_str("906594975575087378705533173710149549372504772722")
                .unwrap()
                .as_u160()
        );
    }
}
