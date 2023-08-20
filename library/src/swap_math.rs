use crate::full_math::{FullMath, FullMathTrait, MathOps};
use crate::num160::*;
use crate::num24::*;
use crate::sqrt_price_math;
use ethnum::{AsU256, I256, U256};

/// @notice Computes the result of swapping some amount in, or amount out, given the parameters of the swap
/// @dev The fee, plus the amount in, will never exceed the amount remaining if the swap's `amountSpecified` is positive
/// @param sqrtRatioCurrentX96 The current sqrt price of the pool
/// @param sqrtRatioTargetX96 The price that cannot be exceeded, from which the direction of the swap is inferred
/// @param liquidity The usable liquidity
/// @param amountRemaining How much input or output amount is remaining to be swapped in/out
/// @param feePips The fee taken from the input amount, expressed in hundredths of a bip
/// @return sqrtRatioNextX96 The price after swapping the amount in/out, not to exceed the price target
/// @return amountIn The amount to be swapped in, of either token0 or token1, based on the direction of the swap
/// @return amountOut The amount to be received, of either token0 or token1, based on the direction of the swap
/// @return feeAmount The amount of input that will be taken as a fee
pub fn compute_swap_step(
    sqrt_ratio_current_x96: U160,
    sqrt_ratio_target_x96: U160,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: U24,
) -> (U160, U256, U256, U256) {
    let zero_for_one = sqrt_ratio_current_x96 >= sqrt_ratio_target_x96;
    let exact_in = amount_remaining >= 0;

    let sqrt_ratio_next_x96;
    let mut amount_in: U256 = U256::ZERO;
    let mut amount_out: U256 = U256::ZERO;
    let fee_amount: U256;

    if exact_in {
        let amount_remaining_less_fee = FullMath::mul_div(
            amount_remaining.as_u256(),
            U256::new((1_000_000 - fee_pips) as u128),
            1_000_000.as_u256(),
        );
        amount_in = if zero_for_one {
            sqrt_price_math::get_amount0_delta(
                sqrt_ratio_target_x96,
                sqrt_ratio_current_x96,
                liquidity,
                true,
            )
        } else {
            sqrt_price_math::get_amount1_delta(
                sqrt_ratio_current_x96,
                sqrt_ratio_target_x96,
                liquidity,
                true,
            )
        };

        if amount_remaining_less_fee >= amount_in {
            sqrt_ratio_next_x96 = sqrt_ratio_target_x96;
        } else {
            sqrt_ratio_next_x96 = sqrt_price_math::get_next_sqrt_price_from_input(
                sqrt_ratio_current_x96,
                liquidity,
                amount_remaining_less_fee,
                zero_for_one,
            );
        }
    } else {
        amount_out = if zero_for_one {
            sqrt_price_math::get_amount1_delta(
                sqrt_ratio_target_x96,
                sqrt_ratio_current_x96,
                liquidity,
                false,
            )
        } else {
            sqrt_price_math::get_amount0_delta(
                sqrt_ratio_current_x96,
                sqrt_ratio_target_x96,
                liquidity,
                false,
            )
        };

        if (I256::ZERO - amount_remaining).as_u256() >= amount_out {
            sqrt_ratio_next_x96 = sqrt_ratio_target_x96;
        } else {
            sqrt_ratio_next_x96 = sqrt_price_math::get_next_sqrt_price_from_output(
                sqrt_ratio_current_x96,
                liquidity,
                (I256::ZERO - amount_remaining).as_u256(),
                zero_for_one,
            );
        }
    }

    let max = sqrt_ratio_target_x96 == sqrt_ratio_next_x96;
    // get the input/output amounts
    if zero_for_one {
        amount_in = if max && exact_in {
            amount_in
        } else {
            sqrt_price_math::get_amount0_delta(
                sqrt_ratio_next_x96,
                sqrt_ratio_current_x96,
                liquidity,
                true,
            )
        };

        amount_out = if max && !exact_in {
            amount_out
        } else {
            sqrt_price_math::get_amount1_delta(
                sqrt_ratio_next_x96,
                sqrt_ratio_current_x96,
                liquidity,
                false,
            )
        };
    } else {
        amount_in = if max && exact_in {
            amount_in
        } else {
            sqrt_price_math::get_amount1_delta(
                sqrt_ratio_current_x96,
                sqrt_ratio_next_x96,
                liquidity,
                true,
            )
        };

        amount_out = if max && !exact_in {
            amount_out
        } else {
            sqrt_price_math::get_amount0_delta(
                sqrt_ratio_current_x96,
                sqrt_ratio_next_x96,
                liquidity,
                false,
            )
        };
    }

    // cap the output amount to not exceed the remaining output amount
    if !exact_in && amount_out > (I256::ZERO - amount_remaining).as_u256() {
        amount_out = (I256::ZERO - amount_remaining).as_u256();
    }

    if exact_in && sqrt_ratio_next_x96 != sqrt_ratio_target_x96 {
        // we didn't reach the target, so take the remainder of the maximum input as fee
        fee_amount = amount_remaining.as_u256() - amount_in;
    } else {
        fee_amount = FullMath::mul_div_rounding_up(
            amount_in,
            fee_pips.as_u256(),
            (1_000_000 - fee_pips).as_u256(),
        );
    }

    (sqrt_ratio_next_x96, amount_in, amount_out, fee_amount)
}

#[cfg(test)]
mod tests {
    use crate::full_math::{FullMath, FullMathTrait, MathOps};
    use crate::num24::{AsU24, U24};
    use crate::sqrt_price_math::{get_next_sqrt_price_from_input, get_next_sqrt_price_from_output};
    use crate::swap_math::compute_swap_step;
    use crate::utils::{encode_price_sqrt_u128, expand_to_18_decimals};
    use ethnum::{I256, U256};
    use std::ops::Sub;
    use std::str::FromStr;

    #[test]
    fn test_compute_swap_step() {
        // exact amount in that gets capped at price target in one for zero
        {
            let price = encode_price_sqrt_u128(1, 1);
            let price_target = encode_price_sqrt_u128(101, 100);
            let liquidity = expand_to_18_decimals(2);
            let amount = expand_to_18_decimals(1).as_i256();
            let fee = 600_u128.as_u24();
            let zero_for_one = false;

            let (sqrt_q, amount_in, amount_out, fee_amount) =
                compute_swap_step(price, price_target, liquidity.as_u128(), amount, fee);

            assert_eq!(amount_in, U256::new(9975124224178055));
            assert_eq!(fee_amount, U256::new(5988667735148));
            assert_eq!(amount_out, U256::new(9925619580021728));
            assert!(amount_in.add(fee_amount).as_i256() < amount);

            let price_after_whole_input_amount = get_next_sqrt_price_from_input(
                price,
                liquidity.as_u128(),
                amount.as_u256(),
                zero_for_one,
            );

            // price is capped at price target
            assert_eq!(sqrt_q, price_target);
            // price is less than price after whole input amount
            assert!(sqrt_q < price_after_whole_input_amount);
        }

        // exact amount out that gets capped at price target in one for zero
        {
            let price = encode_price_sqrt_u128(1, 1);
            let price_target = encode_price_sqrt_u128(101, 100);
            let liquidity = expand_to_18_decimals(2);
            let amount = expand_to_18_decimals(1).as_i256() * I256::from(-1);
            let fee = 600_u128.as_u24();
            let zero_for_one = false;

            let (sqrt_q, amount_in, amount_out, fee_amount) =
                compute_swap_step(price, price_target, liquidity.as_u128(), amount, fee);

            assert_eq!(amount_in, U256::new(9975124224178055));
            assert_eq!(fee_amount, U256::new(5988667735148));
            assert_eq!(amount_out, U256::new(9925619580021728));
            assert!(amount_out < (amount * I256::from(-1)).as_u256());

            let price_after_whole_output_amount = get_next_sqrt_price_from_output(
                price,
                liquidity.as_u128(),
                (amount * I256::from(-1)).as_u256(),
                zero_for_one,
            );

            assert_eq!(sqrt_q, price_target);
            assert!(sqrt_q < price_after_whole_output_amount);
        }
        // exact amount in that is fully spent in one for zero
        {
            let price = encode_price_sqrt_u128(1, 1);
            let price_target = encode_price_sqrt_u128(1000, 100);
            let liquidity = expand_to_18_decimals(2);
            let amount = expand_to_18_decimals(1).as_i256();
            let fee = 600_u128.as_u24();
            let zero_for_one = false;

            let (sqrt_q, amount_in, amount_out, fee_amount) =
                compute_swap_step(price, price_target, liquidity.as_u128(), amount, fee);

            assert_eq!(amount_in, U256::new(999400000000000000));
            assert_eq!(fee_amount, U256::new(600000000000000));
            assert_eq!(amount_out, U256::new(666399946655997866));
            assert_eq!(amount_in.add(fee_amount), amount.as_u256());

            let price_after_whole_input_amount_less_fee = get_next_sqrt_price_from_input(
                price,
                liquidity.as_u128(),
                amount.sub(fee_amount.as_i256()).as_u256(),
                zero_for_one,
            );

            assert!(sqrt_q < price_target);
            assert_eq!(sqrt_q, price_after_whole_input_amount_less_fee);
        }
        // exact amount out that is fully received in one for zero
        {
            let price = encode_price_sqrt_u128(1, 1);
            let price_target = encode_price_sqrt_u128(10000, 100);
            let liquidity = expand_to_18_decimals(2);
            let amount = expand_to_18_decimals(1).as_i256() * I256::new(-1);
            let fee = 600_u128.as_u24();
            let zero_for_one = false;

            let (sqrt_q, amount_in, amount_out, fee_amount) =
                compute_swap_step(price, price_target, liquidity.as_u128(), amount, fee);

            assert_eq!(amount_in, U256::new(2000000000000000000));
            assert_eq!(fee_amount, U256::new(1200720432259356));
            assert_eq!(amount_out, (amount * I256::new(-1)).as_u256());

            let price_after_whole_output_amount = get_next_sqrt_price_from_output(
                price,
                liquidity.as_u128(),
                (amount * I256::from(-1)).as_u256(),
                zero_for_one,
            );

            assert!(sqrt_q < price_target);
            assert_eq!(sqrt_q, price_after_whole_output_amount);
        }
        // amount out is capped at the desired amount out
        {
            let price = U256::from_str("417332158212080721273783715441582").unwrap();
            let price_target = U256::from_str("1452870262520218020823638996").unwrap();
            let liquidity = U256::from_str("159344665391607089467575320103").unwrap();
            let amount = I256::new(-1);
            let fee = 1_u128.as_u24();

            let (sqrt_q, amount_in, amount_out, fee_amount) =
                compute_swap_step(price, price_target, liquidity.as_u128(), amount, fee);

            assert_eq!(amount_in, U256::new(1));
            assert_eq!(fee_amount, U256::new(1));
            assert_eq!(amount_out, U256::new(1));
            assert_eq!(
                sqrt_q,
                U256::from_str("417332158212080721273783715441581").unwrap()
            );
        }

        // target price of 1 uses partial input amount
        {
            let price = U256::from_str("2").unwrap();
            let price_target = U256::from_str("1").unwrap();
            let liquidity = U256::from_str("1").unwrap();
            let amount = I256::from_str("3915081100057732413702495386755767").unwrap();
            let fee = 1_u128.as_u24();

            let (sqrt_q, amount_in, amount_out, fee_amount) =
                compute_swap_step(price, price_target, liquidity.as_u128(), amount, fee);

            assert_eq!(
                amount_in,
                U256::from_str("39614081257132168796771975168").unwrap()
            );
            assert_eq!(
                fee_amount,
                U256::from_str("39614120871253040049813").unwrap()
            );
            assert!(
                amount_in.add(fee_amount)
                    <= U256::from_str("3915081100057732413702495386755767").unwrap()
            );
            assert_eq!(amount_out, U256::from_str("0").unwrap());
            assert_eq!(sqrt_q, U256::from_str("1").unwrap());
        }

        // entire input amount taken as fee
        {
            let price = U256::from_str("2413").unwrap();
            let price_target = U256::from_str("79887613182836312").unwrap();
            let liquidity = U256::from_str("1985041575832132834610021537970").unwrap();
            let amount = I256::from_str("10").unwrap();
            let fee = 1872_u128.as_u24();

            let (sqrt_q, amount_in, amount_out, fee_amount) =
                compute_swap_step(price, price_target, liquidity.as_u128(), amount, fee);
            assert_eq!(amount_in, U256::from_str("0").unwrap());
            assert_eq!(fee_amount, U256::from_str("10").unwrap());
            assert_eq!(amount_out, U256::from_str("0").unwrap());
            assert_eq!(sqrt_q, U256::from_str("2413").unwrap());
        }

        // handles intermediate insufficient liquidity in zero for one exact output case
        {
            let sqrt_p = U256::from_str("20282409603651670423947251286016").unwrap();
            let sqrt_p_target = FullMath::mul_div(sqrt_p, U256::new(11), U256::new(10));
            let liquidity = U256::new(1024);
            // virtual reserves of one are only 4
            // https://www.wolframalpha.com/input/?i=1024+%2F+%2820282409603651670423947251286016+%2F+2**96%29
            let amount_remaining = I256::from(-4);
            let fee_pips = 3000_u128.as_u24();

            let (sqrt_q, amount_in, amount_out, fee_amount) = compute_swap_step(
                sqrt_p,
                sqrt_p_target,
                liquidity.as_u128(),
                amount_remaining,
                fee_pips,
            );

            assert_eq!(amount_out, U256::from_str("0").unwrap());
            assert_eq!(sqrt_q, sqrt_p_target);
            assert_eq!(amount_in, U256::from_str("26215").unwrap());
            assert_eq!(fee_amount, U256::from_str("79").unwrap());
        }

        // handles intermediate insufficient liquidity in one for zero exact output case
        {
            let sqrt_p = U256::from_str("20282409603651670423947251286016").unwrap();
            let sqrt_p_target = FullMath::mul_div(sqrt_p, U256::new(9), U256::new(10));
            let liquidity = U256::new(1024);
            // virtual reserves of zero are only 262144
            // https://www.wolframalpha.com/input/?i=1024+*+%2820282409603651670423947251286016+%2F+2**96%29
            let amount_remaining = I256::from(-263000);
            let fee_pips = 3000_u128.as_u24();

            let (sqrt_q, amount_in, amount_out, fee_amount) = compute_swap_step(
                sqrt_p,
                sqrt_p_target,
                liquidity.as_u128(),
                amount_remaining,
                fee_pips,
            );

            assert_eq!(amount_out, U256::from_str("26214").unwrap());
            assert_eq!(sqrt_q, sqrt_p_target);
            assert_eq!(amount_in, U256::from_str("1").unwrap());
            assert_eq!(fee_amount, U256::from_str("1").unwrap());
        }
    }
}
