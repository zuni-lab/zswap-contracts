use ethnum::I256;

use crate::fixed_point_96;
use crate::full_math::{FullMath, FullMathTrait, MathOps};
use crate::num160::{AsU160, Num160Trait, U160};
use crate::num256::{AsI256, U256};

/// @notice Gets the next sqrt price given a delta of token0
/// @dev Always rounds up, because in the exact output case (increasing price) we need to move the price at least
/// far enough to get the desired output amount, and in the exact input case (decreasing price) we need to move the
/// price less in order to not send too much output.
/// The most precise formula for this is liquidity * sqrtPX96 / (liquidity +- amount * sqrtPX96),
/// if this is impossible because of overflow, we calculate liquidity / (liquidity / sqrtPX96 +- amount).
/// @param sqrtPX96 The starting price, i.e. before accounting for the token0 delta
/// @param liquidity The amount of usable liquidity
/// @param amount How much of token0 to add or remove from virtual reserves
/// @param add Whether to add or remove the amount of token0
/// @return The price after adding or removing amount, depending on add
pub fn get_next_sqrt_price_from_amount0_rounding_up(
    sqrt_px96: U160,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> U160 {
    // we short circuit amount == 0 because the result is otherwise not guaranteed to equal the input price
    if amount == U256::zero() {
        return sqrt_px96;
    }

    let numerator1 = U256::from(liquidity) << fixed_point_96::RESOLUTION;

    if add {
        if let Some(product) = amount.checked_mul(sqrt_px96) {
            let denominator = MathOps::add(numerator1, product);
            if denominator >= numerator1 {
                // always fits in 160 bits
                // always fits in 160 bits
                return (FullMath::mul_div_rounding_up(numerator1, sqrt_px96, denominator) as U160)
                    .as_u160();
            }
        }
        FullMath::unsafe_div_rounding_up(numerator1, (numerator1 / sqrt_px96).add(amount)).as_u160()
    } else {
        if let Some(product) = amount.checked_mul(sqrt_px96) {
            if numerator1 > product {
                if let Some(denominator) = numerator1.checked_sub(product) {
                    return FullMath::mul_div_rounding_up(numerator1, sqrt_px96, denominator)
                        .as_u160();
                }
            }
        }
        panic!("Denominator underflow");
    }
}

/// @notice Gets the next sqrt price given a delta of token1
/// @dev Always rounds down, because in the exact output case (decreasing price) we need to move the price at least
/// far enough to get the desired output amount, and in the exact input case (increasing price) we need to move the
/// price less in order to not send too much output.
/// The formula we compute is within <1 wei of the lossless version: sqrtPX96 +- amount / liquidity
/// @param sqrtPX96 The starting price, i.e., before accounting for the token1 delta
/// @param liquidity The amount of usable liquidity
/// @param amount How much of token1 to add, or remove, from virtual reserves
/// @param add Whether to add, or remove, the amount of token1
/// @return The price after adding or removing `amount`
pub fn get_next_sqrt_price_from_amount1_rounding_down(
    sqrt_px96: U160,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> U160 {
    // if we're adding (subtracting), rounding down requires rounding the quotient down (up)
    // in both cases, avoid a mulDiv for most inputs
    if add {
        let quotient = if amount <= (U256::one() << 160) - U256::one() {
            (amount << fixed_point_96::RESOLUTION) / liquidity
        } else {
            FullMath::mul_div(amount, fixed_point_96::get_q96(), U256::from(liquidity))
        };

        (sqrt_px96.add160(quotient) as U160).as_u160()
    } else {
        let quotient = if amount <= (U256::one() << 160) - U256::one() {
            FullMath::unsafe_div_rounding_up(
                amount << fixed_point_96::RESOLUTION,
                U256::from(liquidity),
            )
        } else {
            FullMath::mul_div_rounding_up(amount, fixed_point_96::get_q96(), U256::from(liquidity))
        };

        if sqrt_px96 > quotient {
            // always fits 160 bits
            sqrt_px96.sub(quotient).as_u160()
        } else {
            panic!("Sqrt price must be greater than quotient");
        }
    }
}

/// @notice Gets the next sqrt price given an input amount of token0 or token1
/// @dev Throws if price or liquidity are 0, or if the next price is out of bounds
/// @param sqrtPX96 The starting price, i.e., before accounting for the input amount
/// @param liquidity The amount of usable liquidity
/// @param amountIn How much of token0, or token1, is being swapped in
/// @param zeroForOne Whether the amount in is token0 or token1
/// @return sqrtQX96 The price after adding the input amount to token0 or token1
pub fn get_next_sqrt_price_from_input(
    sqrt_px96: U160,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> U160 {
    assert!(sqrt_px96 > U160::zero() && liquidity > 0);

    if zero_for_one {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_in, true)
    }
}

/// @notice Gets the next sqrt price given an output amount of token0 or token1
/// @dev Throws if price or liquidity are 0 or the next price is out of bounds
/// @param sqrtPX96 The starting price before accounting for the output amount
/// @param liquidity The amount of usable liquidity
/// @param amountOut How much of token0, or token1, is being swapped out
/// @param zeroForOne Whether the amount out is token0 or token1
/// @return sqrtQX96 The price after removing the output amount of token0 or token1
pub fn get_next_sqrt_price_from_output(
    sqrt_px96: U160,
    liquidity: u128,
    amount_out: U256,
    zero_for_one: bool,
) -> U160 {
    assert!(sqrt_px96 > U160::zero() && liquidity > 0);

    if zero_for_one {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_out, false)
    }
}

/// @notice Gets the amount0 delta between two prices
/// @dev Calculates liquidity / sqrt(lower) - liquidity / sqrt(upper),
/// i.e. liquidity * (sqrt(upper) - sqrt(lower)) / (sqrt(upper) * sqrt(lower))
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The amount of usable liquidity
/// @param roundUp Whether to round the amount up or down
/// @return amount0 Amount of token0 required to cover a position of size liquidity between the two passed prices
pub fn get_amount0_delta(
    _sqrt_ratio_a_x96: U160,
    _sqrt_ratio_b_x96: U160,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    let (mut sqrt_ratio_a_x96, mut sqrt_ratio_b_x96) = (_sqrt_ratio_a_x96, _sqrt_ratio_b_x96);

    if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
        std::mem::swap(&mut sqrt_ratio_a_x96, &mut sqrt_ratio_b_x96);
    }

    let numerator1 = U256::from(liquidity) << fixed_point_96::RESOLUTION;
    let numerator2 = sqrt_ratio_b_x96.sub(sqrt_ratio_a_x96);

    assert!(sqrt_ratio_a_x96 > U160::zero());

    if round_up {
        FullMath::unsafe_div_rounding_up(
            FullMath::mul_div_rounding_up(numerator1, numerator2, sqrt_ratio_b_x96),
            sqrt_ratio_a_x96,
        )
    } else {
        FullMath::mul_div(numerator1, numerator2, sqrt_ratio_b_x96) / sqrt_ratio_a_x96
    }
}

/// @notice Gets the amount1 delta between two prices
/// @dev Calculates liquidity * (sqrt(upper) - sqrt(lower))
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The amount of usable liquidity
/// @param roundUp Whether to round the amount up, or down
/// @return amount1 Amount of token1 required to cover a position of size liquidity between the two passed prices
pub fn get_amount1_delta(
    _sqrt_ratio_a_x96: U160,
    _sqrt_ratio_b_x96: U160,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    let (mut sqrt_ratio_a_x96, mut sqrt_ratio_b_x96) = (_sqrt_ratio_a_x96, _sqrt_ratio_b_x96);

    if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
        std::mem::swap(&mut sqrt_ratio_a_x96, &mut sqrt_ratio_b_x96);
    }

    if round_up {
        FullMath::mul_div_rounding_up(
            U256::from(liquidity),
            sqrt_ratio_b_x96.sub(sqrt_ratio_a_x96),
            fixed_point_96::get_q96(),
        )
    } else {
        FullMath::mul_div(
            U256::from(liquidity),
            sqrt_ratio_b_x96.sub(sqrt_ratio_a_x96),
            fixed_point_96::get_q96(),
        )
    }
}

/// @notice Helper that gets signed token0 delta
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The change in liquidity for which to compute the amount0 delta
/// @return amount0 Amount of token0 corresponding to the passed liquidityDelta between the two prices
pub fn get_amount0_delta_signed(
    sqrt_ratio_a_x96: U160,
    sqrt_ratio_b_x96: U160,
    liquidity: i128,
) -> I256 {
    if liquidity < 0 {
        -get_amount0_delta(
            sqrt_ratio_a_x96,
            sqrt_ratio_b_x96,
            -liquidity as u128,
            false,
        )
        .as_i256()
    } else {
        get_amount0_delta(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity as u128, true).as_i256()
    }
}

/// @notice Helper that gets signed token1 delta
/// @param sqrtRatioAX96 A sqrt price
/// @param sqrtRatioBX96 Another sqrt price
/// @param liquidity The change in liquidity for which to compute the amount1 delta
/// @return amount1 Amount of token1 corresponding to the passed liquidityDelta between the two prices
pub fn get_amount1_delta_signed(
    sqrt_ratio_a_x96: U160,
    sqrt_ratio_b_x96: U160,
    liquidity: i128,
) -> I256 {
    if liquidity < 0 {
        -get_amount1_delta(
            sqrt_ratio_a_x96,
            sqrt_ratio_b_x96,
            -liquidity as u128,
            false,
        )
        .as_i256()
    } else {
        get_amount1_delta(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity as u128, true).as_i256()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::full_math::MathOps;
    use crate::num160::AsU160;
    use crate::utils;
    use std::ops::Shl;
    use std::panic;

    #[test]
    fn test_get_next_sqrt_price_from_input() {
        // Fails if price is zero
        assert!(panic::catch_unwind(|| {
            get_next_sqrt_price_from_input(
                U160::zero(),
                0 as u128,
                MathOps::div(utils::expand_to_18_decimals(1), U256::from(10)),
                false,
            );
        })
        .is_err(),);

        // Fails if liquidity is zero
        assert!(panic::catch_unwind(|| {
            get_next_sqrt_price_from_input(
                U160::one(),
                0 as u128,
                MathOps::div(utils::expand_to_18_decimals(1), U256::from(10)),
                true,
            );
        })
        .is_err(),);

        // Fails if input amount overflows the price
        {
            let price = U256::from(2)
                .pow(U256::from(160))
                .sub(U256::one())
                .as_u160();
            let liquidity = 1024;
            let amount_in = U256::from(1024);
            assert!(panic::catch_unwind(|| {
                get_next_sqrt_price_from_input(price, liquidity, amount_in, false);
            })
            .is_err(),);
        }

        // Returns input price if amount in is zero and zeroForOne = true
        {
            let price = utils::encode_price_sqrt_u128(1, 1);
            assert_eq!(
                get_next_sqrt_price_from_input(
                    price,
                    MathOps::div(utils::expand_to_18_decimals(1), U256::from(10)).as_u128(),
                    U256::zero(),
                    true
                ),
                price,
            );
        }

        // Returns input price if amount in is zero and zeroForOne = false
        {
            let price = utils::encode_price_sqrt_u128(1, 1);
            assert_eq!(
                get_next_sqrt_price_from_input(
                    price,
                    MathOps::div(utils::expand_to_18_decimals(1), U256::from(10)).as_u128(),
                    U256::zero(),
                    false
                ),
                price,
            );
        }

        // Returns the minimum price for max inputs
        {
            let sqrt_p = U256::from(2).pow(U256::from(160)) - U256::one();
            let liquidity = U256::from(u128::MAX);

            let max_amount_no_overflow = U256::MAX - liquidity.shl(U256::from(96)).div(sqrt_p);

            assert_eq!(
                get_next_sqrt_price_from_input(
                    sqrt_p,
                    liquidity.as_u128(),
                    max_amount_no_overflow,
                    true
                ),
                U256::one()
            );
        }

        // Input amount of 0.1 token1
        {
            let sqrt_q = get_next_sqrt_price_from_input(
                utils::encode_price_sqrt_u128(1, 1),
                utils::expand_to_18_decimals(1).as_u128(),
                MathOps::div(utils::expand_to_18_decimals(1), U256::from(10)),
                false,
            );
            assert_eq!(
                sqrt_q,
                U256::from_dec_str("87150978765690771352898345369").unwrap(),
            );
        }

        // Input amount of 0.1 token0
        {
            let sqrt_q = get_next_sqrt_price_from_input(
                utils::encode_price_sqrt_u128(1, 1),
                utils::expand_to_18_decimals(1).as_u128(),
                MathOps::div(utils::expand_to_18_decimals(1), U256::from(10)),
                true,
            );
            assert_eq!(
                sqrt_q,
                U256::from_dec_str("72025602285694852357767227579").unwrap()
            );
        }

        // amountIn > type(uint96).max and zeroForOne = true
        assert_eq!(
            get_next_sqrt_price_from_input(
                utils::encode_price_sqrt_u128(1, 1),
                utils::expand_to_18_decimals(10).as_u128(),
                U256::from(2).pow(U256::from(100)),
                true,
            ),
            U256::from_dec_str("624999999995069620").unwrap()
        );

        // Can return 1 with enough amountIn and zeroForOne = true
        assert_eq!(
            get_next_sqrt_price_from_input(
                utils::encode_price_sqrt_u128(1, 1),
                1,
                U256::MAX.div(U256::from(2)),
                true,
            ),
            U256::one()
        );
    }

    #[test]
    fn test_get_next_sqrt_price_from_output() {
        //fails if price is zero
        assert!(panic::catch_unwind(|| {
            get_next_sqrt_price_from_output(
                U256::zero(),
                0,
                utils::expand_to_18_decimals(1).div(U256::from(10)),
                false,
            );
        })
        .is_err());

        // fails if liquidity is zero
        assert!(panic::catch_unwind(|| {
            get_next_sqrt_price_from_output(
                U256::one(),
                0,
                utils::expand_to_18_decimals(1).div(U256::from(10)),
                true,
            );
        })
        .is_err());

        // fails if output amount is exactly the virtual reserves of token0
        assert!(panic::catch_unwind(|| {
            let price = U256::from_dec_str("20282409603651670423947251286016").unwrap();
            let liquidity = 1024 as u128;
            let amount_out = U256::from(4);
            get_next_sqrt_price_from_output(price, liquidity, amount_out, false);
        })
        .is_err());

        // fails if output amount is greater than virtual reserves of token1
        assert!(panic::catch_unwind(|| {
            let price = U256::from_dec_str("20282409603651670423947251286016").unwrap();
            let liquidity = 1024 as u128;
            let amount_out = U256::from(262145);
            get_next_sqrt_price_from_output(price, liquidity, amount_out, true);
        })
        .is_err());

        // fails if output amount is exactly the virtual reserves of token1
        assert!(panic::catch_unwind(|| {
            let price = U256::from_dec_str("20282409603651670423947251286016").unwrap();
            let liquidity = 1024 as u128;
            let amount_out = U256::from(262144);
            get_next_sqrt_price_from_output(price, liquidity, amount_out, true);
        })
        .is_err());

        //succeeds if output amount is just less than the virtual reserves of token1
        {
            let price = U256::from_dec_str("20282409603651670423947251286016").unwrap();
            let liquidity = 1024 as u128;
            let amount_out = U256::from(262143);
            assert_eq!(
                get_next_sqrt_price_from_output(price, liquidity, amount_out, true),
                U160::from_dec_str("77371252455336267181195264").unwrap()
            );
        }
        // puzzling echidna test
        assert!(panic::catch_unwind(|| {
            let price = U256::from_dec_str("20282409603651670423947251286016").unwrap();
            let liquidity = 1024 as u128;
            let amount_out = U256::from(4);
            get_next_sqrt_price_from_output(price, liquidity, amount_out, false);
        })
        .is_err());

        // returns input price if amount in is zero and zeroForOne = true
        {
            let price = utils::encode_price_sqrt_u128(1, 1);
            let liquidity = utils::expand_to_18_decimals(1)
                .div(U256::from(10))
                .as_u128();
            let amount_out = U256::from(0);
            assert_eq!(
                get_next_sqrt_price_from_output(price, liquidity, amount_out, true),
                price
            );
        }

        // returns input price if amount in is zero and zeroForOne = false
        {
            let price = utils::encode_price_sqrt_u128(1, 1);
            let liquidity = utils::expand_to_18_decimals(1)
                .div(U256::from(10))
                .as_u128();
            let amount_out = U256::from(0);
            assert_eq!(
                get_next_sqrt_price_from_output(price, liquidity, amount_out, false),
                price
            );
        }

        // returns input price if amount in is zero and zeroForOne = false
        {
            let price = utils::encode_price_sqrt_u128(1, 1);
            let liquidity = utils::expand_to_18_decimals(1).as_u128();
            let amount_out = utils::expand_to_18_decimals(1).div(U256::from(10));
            assert_eq!(
                get_next_sqrt_price_from_output(price, liquidity, amount_out, false),
                U160::from_dec_str("88031291682515930659493278152").unwrap()
            );
        }

        // output amount of 0.1 token1
        {
            let price = utils::encode_price_sqrt_u128(1, 1);
            let liquidity = utils::expand_to_18_decimals(1).as_u128();
            let amount_out = utils::expand_to_18_decimals(1).div(U256::from(10));
            assert_eq!(
                get_next_sqrt_price_from_output(price, liquidity, amount_out, true),
                U160::from_dec_str("71305346262837903834189555302").unwrap()
            );
        }

        // reverts if amountOut is impossible in zero for one direction
        assert!(panic::catch_unwind(|| {
            let price = utils::encode_price_sqrt_u128(1, 1);
            let liquidity = 1 as u128;
            let amount_out = U256::MAX;
            get_next_sqrt_price_from_output(price, liquidity, amount_out, true);
        })
        .is_err());

        // reverts if amountOut is impossible in one for zero direction
        assert!(panic::catch_unwind(|| {
            let price = utils::encode_price_sqrt_u128(1, 1);
            let liquidity = 1 as u128;
            let amount_out = U256::MAX;
            get_next_sqrt_price_from_output(price, liquidity, amount_out, false);
        })
        .is_err());
    }

    #[test]
    fn test_get_amount0_delta() {
        // returns 0 if liquidity is 0
        assert_eq!(
            get_amount0_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(2, 1),
                0,
                true
            ),
            U256::zero()
        );

        // returns 0 if prices are equal
        assert_eq!(
            get_amount0_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(1, 1),
                0,
                true
            ),
            U256::zero()
        );

        {
            // returns 0.1 amount0 for price of 1 to 1.21
            let amount0 = get_amount0_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(121, 100),
                utils::expand_to_18_decimals(1).as_u128(),
                true,
            );
            assert_eq!(amount0, U256::from(90909090909090910 as u128));

            let amount0_rounded_down = get_amount0_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(121, 100),
                utils::expand_to_18_decimals(1).as_u128(),
                false,
            );
            assert_eq!(amount0_rounded_down, amount0.sub(U256::one()));
        }

        {
            // works for prices that overflow
            let amount0_up = get_amount0_delta(
                utils::encode_price_sqrt_u128(2_u128.pow(90), 1),
                utils::encode_price_sqrt_u128(2_u128.pow(96), 1),
                utils::expand_to_18_decimals(1).as_u128(),
                true,
            );
            let amount0_down = get_amount0_delta(
                utils::encode_price_sqrt_u128(2_u128.pow(90), 1),
                utils::encode_price_sqrt_u128(2_u128.pow(96), 1),
                utils::expand_to_18_decimals(1).as_u128(),
                false,
            );

            assert_eq!(amount0_up, amount0_down.add(U256::one()));
        }
        // gas cost tests ...
    }

    #[test]
    fn test_get_amount1_delta() {
        // returns 0 if liquidity is 0
        assert_eq!(
            get_amount1_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(2, 1),
                0,
                true
            ),
            U256::zero()
        );

        // returns 0 if prices are equal
        assert_eq!(
            get_amount0_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(1, 1),
                0,
                true
            ),
            U256::zero()
        );

        // returns 0.1 amount1 for price of 1 to 1.21
        {
            let amount1 = get_amount1_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(121, 100),
                utils::expand_to_18_decimals(1).as_u128(),
                true,
            );
            assert_eq!(amount1, U256::from(100000000000000000 as u128));

            let amount1_rounded_down = get_amount1_delta(
                utils::encode_price_sqrt_u128(1, 1),
                utils::encode_price_sqrt_u128(121, 100),
                utils::expand_to_18_decimals(1).as_u128(),
                false,
            );
            assert_eq!(amount1_rounded_down, amount1.sub(U256::one()));
        }
    }

    #[test]
    fn test_swap_computation() {
        // sqrtP * sqrtQ overflows
        // getNextSqrtPriceInvariants(1025574284609383690408304870162715216695788925244,50015962439936049619261659728067971248,406,true)
        let sqrt_p =
            U256::from_dec_str("1025574284609383690408304870162715216695788925244").unwrap();
        let liquidity = 50015962439936049619261659728067971248_u128;
        let zero_for_one = true;
        let amount_in = U256::from(406);

        let sqrt_q = get_next_sqrt_price_from_input(sqrt_p, liquidity, amount_in, zero_for_one);
        assert_eq!(
            sqrt_q,
            U160::from_dec_str("1025574284609383582644711336373707553698163132913").unwrap()
        );

        let amount0_delta = get_amount0_delta(sqrt_q, sqrt_p, liquidity as u128, true);
        assert_eq!(amount0_delta, amount_in);

        // The remaining assertions related to gas cost can also be adapted in a similar manner.
    }
}
