use ethnum::I256;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    #[derive(BorshDeserialize, BorshSerialize)]
    pub struct U256(4);
}

pub trait AsI256 {
    fn as_i256(&self) -> I256;
}

impl AsI256 for U256 {
    fn as_i256(&self) -> I256 {
        I256::from_str_radix(&self.to_string(), 10).unwrap()
    }
}

pub trait ToU256 {
    fn to_u256(&self) -> U256;
}

impl ToU256 for I256 {
    fn to_u256(&self) -> U256 {
        U256::from_str_radix(&self.to_string(), 10).unwrap()
    }
}
