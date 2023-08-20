use ethnum::{AsU256, I256, U256};

use super::full_math::FullMath;
use super::num160::U160;
use super::num24::U24;
use crate::full_math::FullMathTrait;
use crate::sqrt_price_math;

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
        fee_amount = (I256::ZERO - amount_remaining).as_u256() - amount_in;
    } else {
        fee_amount = FullMath::mul_div(
            amount_in,
            fee_pips.as_u256(),
            (1_000_000 - fee_pips).as_u256(),
        );
    }

    (sqrt_ratio_next_x96, amount_in, amount_out, fee_amount)
}

#[cfg(test)]
mod tests {
    #[test]
    fn compute_swap_step() {
        // assert_eq!(SwapMath::add_delta(1, 1), 2);
        // // 2**128-15 + 15 overflows
        // assert!(panic::catch_unwind(|| {
        //   SwapMath::add_delta((U256::new(2).pow(128) - U256::new(15)).as_u128(), 15);
        // }).is_err());
    }
}
