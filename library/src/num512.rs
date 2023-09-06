#![allow(clippy::assign_op_pattern)]

use uint::construct_uint;

construct_uint! {
    /// 512-bit unsigned integer.
    pub struct U512(8);
}
