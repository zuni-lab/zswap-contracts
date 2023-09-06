use crate::fixed_point_96;
use crate::num256::U256;
use crate::num512::U512;

/// @notice Add a signed liquidity delta to liquidity and revert if it overflows or underflows
/// @param x The liquidity before change
/// @param y The delta by which liquidity should be changed
/// @return z The liquidity delta
pub fn add_delta(x: u128, y: i128) -> u128 {
    let z;
    if y < 0 {
        z = x - ((0 - y) as u128);
        assert!(z < x);
    } else {
        z = x + (y as u128);
        assert!(z >= x);
    }
    z
}

/// $L = \frac{\Delta x \sqrt{P_u} \sqrt{P_l}}{\Delta \sqrt{P}}$
pub fn get_liquidity_for_amount_0(
    _sqrt_price_a_x96: U256,
    _sqrt_price_b_x96: U256,
    amount_0: u128,
) -> u128 {
    let (sqrt_price_a_x96, sqrt_price_b_x96) = if _sqrt_price_a_x96 > _sqrt_price_b_x96 {
        (_sqrt_price_b_x96, _sqrt_price_a_x96)
    } else {
        (_sqrt_price_a_x96, _sqrt_price_b_x96)
    };

    let amount_0 = U512::from_dec_str(&amount_0.to_string()).unwrap();
    let sqrt_price_a_x96 = U512::from_dec_str(&sqrt_price_a_x96.to_string()).unwrap();
    let sqrt_price_b_x96 = U512::from_dec_str(&sqrt_price_b_x96.to_string()).unwrap();
    let fixed_point_96 = U512::from_dec_str(&fixed_point_96::get_q96().to_string()).unwrap();

    let intermediate = (sqrt_price_a_x96 * sqrt_price_b_x96) / fixed_point_96;
    let liquidity = (amount_0 * intermediate) / (sqrt_price_b_x96 - sqrt_price_a_x96);

    // let intermediate = FullMath::mul_div(
    //     sqrt_price_a_x96,
    //     sqrt_price_b_x96,
    //     fixed_point_96::get_q96(),
    // );
    // let liquidity = FullMath::mul_div(
    //     amount_0,
    //     intermediate,
    //     sqrt_price_b_x96.sub(sqrt_price_a_x96),
    // );
    liquidity.as_u128()
}

/// $L = \frac{\Delta y}{\Delta \sqrt{P}}$
pub fn get_liquidity_for_amount_1(
    _sqrt_price_a_x96: U256,
    _sqrt_price_b_x96: U256,
    amount_1: u128,
) -> u128 {
    let (sqrt_price_a_x96, sqrt_price_b_x96) = if _sqrt_price_a_x96 > _sqrt_price_b_x96 {
        (_sqrt_price_b_x96, _sqrt_price_a_x96)
    } else {
        (_sqrt_price_a_x96, _sqrt_price_b_x96)
    };

    let amount_1 = U512::from_dec_str(&amount_1.to_string()).unwrap();
    let sqrt_price_a_x96 = U512::from_dec_str(&sqrt_price_a_x96.to_string()).unwrap();
    let sqrt_price_b_x96 = U512::from_dec_str(&sqrt_price_b_x96.to_string()).unwrap();
    let fixed_point_96 = U512::from_dec_str(&fixed_point_96::get_q96().to_string()).unwrap();
    let liquidity = (amount_1 * fixed_point_96) / (sqrt_price_b_x96 - sqrt_price_a_x96);

    // let liquidity = FullMath::mul_div(
    //     amount_1,
    //     fixed_point_96::get_q96(),
    //     sqrt_price_b_x96.sub(sqrt_price_a_x96),
    // );
    liquidity.as_u128()
}

pub fn get_liquidity_for_amounts(
    sqrt_price_x96: U256,
    _sqrt_price_a_x96: U256,
    _sqrt_price_b_x96: U256,
    amount_0: u128,
    amount_1: u128,
) -> u128 {
    let (sqrt_price_a_x96, sqrt_price_b_x96) = if _sqrt_price_a_x96 > _sqrt_price_b_x96 {
        (_sqrt_price_b_x96, _sqrt_price_a_x96)
    } else {
        (_sqrt_price_a_x96, _sqrt_price_b_x96)
    };

    let liquidity: u128;
    if sqrt_price_x96 <= sqrt_price_a_x96 {
        liquidity = get_liquidity_for_amount_0(sqrt_price_a_x96, sqrt_price_b_x96, amount_0);
    } else if sqrt_price_x96 <= sqrt_price_b_x96 {
        let liquidity_0 = get_liquidity_for_amount_0(sqrt_price_x96, sqrt_price_b_x96, amount_0);
        let liquidity_1 = get_liquidity_for_amount_1(sqrt_price_a_x96, sqrt_price_x96, amount_1);

        liquidity = if liquidity_0 < liquidity_1 {
            liquidity_0
        } else {
            liquidity_1
        };
    } else {
        liquidity = get_liquidity_for_amount_1(sqrt_price_a_x96, sqrt_price_b_x96, amount_1);
    }
    liquidity
}

pub fn add_liquidity(x: u128, y: i128) -> u128 {
    let z: u128 = if y < 0 {
        x - y.unsigned_abs()
    } else {
        x + (y as u128)
    };
    z
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_add_delta() {
        assert_eq!(add_delta(1, 0), 1);
        assert_eq!(add_delta(1, -1), 0);
        assert_eq!(add_delta(1, 1), 2);
        // 2**128-15 + 15 overflows
        assert!(panic::catch_unwind(|| {
            add_delta(
                (U256::from(2).pow(U256::from(128)) - U256::from(15)).as_u128(),
                15,
            );
        })
        .is_err());
        // 0 + -1 underflows
        assert!(panic::catch_unwind(|| {
            add_delta(0, -1);
        })
        .is_err());
        // 3 + -4 underflows
        assert!(panic::catch_unwind(|| {
            add_delta(3, -4);
        })
        .is_err());
    }

    #[test]
    fn test_get_liquidity_for_amount_0() {
        // TODO: @galin-chung-nguyen
    }

    #[test]
    fn test_get_liquidity_for_amount_1() {
        // TODO: @galin-chung-nguyen
    }

    #[test]
    fn test_get_liquidity_for_amounts() {
        let sqrt_price_x96 = U256::from_dec_str("792281450588003167884250659085").unwrap(); // amount_0/amount_1 = 100, tick = 46056
        let sqrt_price_a_x96 = U256::from_dec_str("646922711029656030980122427077").unwrap(); // tick = 42000
        let sqrt_price_b_x96 = U256::from_dec_str("873241221460953509178849710283").unwrap(); // tick = 48000
        let amount_0 = 1_000;
        let amount_1 = 100_000;
        let liquidity = get_liquidity_for_amounts(
            sqrt_price_x96,
            sqrt_price_a_x96,
            sqrt_price_b_x96,
            amount_0,
            amount_1,
        );
        assert_eq!(liquidity, 54505);
    }

    #[test]
    fn test_add_liquidity() {
        // TODO: @galin-chung-nguyen
    }
}
