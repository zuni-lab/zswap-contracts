use ethnum::U256;

pub mod fixed_point_96 {
    use super::*;

    pub const RESOLUTION: u8 = 96;

    pub fn q96() -> U256 {
        U256::ONE.checked_shl(96).unwrap()
    }
}

pub mod fixed_point_128 {
    use super::*;

    pub const RESOLUTION: u8 = 128;

    pub fn q128() -> U256 {
        U256::ONE.checked_shl(128).unwrap()
    }
}
