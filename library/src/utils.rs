use crate::full_math::{FullMath, FullMathTrait};
use crate::num256::U256;

// returns the sqrt price as a 64x96
pub fn encode_price_sqrt(reserve1: U256, reserve0: U256) -> U256 {
    let a = reserve1;
    let b = reserve0;
    let c = U256::one() << U256::from(96);
    // res = sqrt(a/b) * c
    // res^2 <= a / b * c^2
    // res <= (a * c^2 / (res * b))
    let mut l = U256::from(4295128739 as u64);
    let mut r = U256::one() << U256::from(160);
    let mut res = U256::zero();

    while l <= r {
        let mid: U256 = (l + r) >> 1;
        // TODO: @galin-chung-nguyen: use safer formulation since res * b might overflows
        let x = FullMath::mul_div(a, c.pow(U256::from(2)), mid.overflowing_mul(b).0);
        if mid <= x {
            res = mid;
            l = mid + 1;
        } else {
            r = mid - 1;
        }
    }
    res
}

pub fn encode_price_sqrt_u128(reserve1: u128, reserve0: u128) -> U256 {
    encode_price_sqrt(U256::from(reserve1), U256::from(reserve0))
}

pub fn expand_to_18_decimals(amount: u128) -> U256 {
    U256::from(amount) * U256::from(10).pow(U256::from(18))
}
