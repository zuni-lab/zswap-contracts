#![allow(unused_doc_comments)]
#![allow(dead_code)]
#![allow(unused_variables)]

use ethnum::{AsU256, U256};

pub struct FullMath;

pub trait MathOps {
    fn gt(self, other: Self) -> Self;
    fn lt(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;
    fn div(self, other: Self) -> Self;
    fn modulo(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
    fn mulmod(self, other: Self, modulo: Self) -> Self;
    fn unsafe_muldiv(self, other: Self, modulo: Self) -> Self;
    fn addmod(self, other: Self, modulo: Self) -> Self;
}

impl MathOps for U256 {
    fn gt(self, other: Self) -> Self {
        if self > other {
            U256::ONE
        } else {
            U256::ZERO
        }
    }
    fn lt(self, other: Self) -> Self {
        if self < other {
            U256::ONE
        } else {
            U256::ZERO
        }
    }
    fn sub(self, other: Self) -> Self {
        self.overflowing_sub(other).0
    }
    fn add(self, other: Self) -> Self {
        return self.overflowing_add(other).0;
    }
    fn div(self, other: Self) -> Self {
        return self / other;
    }
    fn modulo(self, other: Self) -> Self {
        return self % other;
    }
    fn mul(self, other: Self) -> Self {
        return self.overflowing_mul(other).0;
    } // https://locklessinc.com/articles/256bit_arithmetic/

    // binary multiplication
    fn mulmod(self, other: Self, modulo: Self) -> Self {
        let mut a = self;
        let mut b = other;
        a %= modulo;
        b %= modulo;
        let mut result: U256 = U256::new(0);
        while b > 0 {
            if (b & U256::ONE) == U256::ONE {
                result = MathOps::addmod(result, a, modulo);
            }
            a = MathOps::addmod(a, a, modulo);
            b = b >> 1;
        }

        return result;
    }
    fn unsafe_muldiv(self, other: Self, modulo: Self) -> Self {
        return self * other / modulo;
    }
    fn addmod(self, other: Self, modulo: Self) -> Self {
        let a = self % modulo;
        let b = other % modulo;
        let remaining_a = modulo - a;
        let res = if remaining_a <= b {
            b - remaining_a
        } else {
            a + b
        };
        res
    }
}

/// SPDX-License-Identifier: MIT
///! Contains 512-bit math functions
///! Facilitates multiplication and division that can have overflow of an intermediate value without any loss of precision
///! Handles "phantom overflow" i.e., allows multiplication and division where an intermediate value overflows 256 bits
pub trait FullMathTrait {
    fn mul_div(a: U256, b: U256, denominator: U256) -> U256;
    fn mul_div_rounding_up(a: U256, b: U256, denominator: U256) -> U256;
    fn unsafe_div_rounding_up(x: U256, y: U256) -> U256;
}

impl FullMathTrait for FullMath {
    // Note: https://notes.ethereum.org/@solidity/ryNbZ2xEq

    // @notice Calculates floor(a×b÷denominator) with full precision. Throws if result overflows a uint256 or denominator == 0
    // @param a The multiplicand
    // @param b The multiplier
    // @param denominator The divisor
    // @return result The 256-bit result
    fn mul_div(a: U256, b: U256, mut denominator: U256) -> U256 {
        assert!(denominator > U256::ZERO);
        // 512-bit multiply [prod1 prod0] = a * b
        // Compute the product mod 2**256 and mod 2**256 - 1
        // then use the Chinese Remainder Theorem to reconstruct
        // the 512 bit result. The result is stored in two 256
        // variables such that product = prod1 * 2**256 + prod0
        let prod0: U256; // Least significant 256 bits of the product
        let prod1: U256; // Most significant 256 bits of the product

        let mm = MathOps::mulmod(a, b, !U256::ZERO);
        let mut prod0 = MathOps::mul(a, b);
        let mut prod1 =
            U256::overflowing_sub(U256::overflowing_sub(mm, prod0).0, MathOps::lt(mm, prod0)).0;

        // Handle non-overflow cases, 256 by 256 division
        if prod1 == 0 {
            assert!(denominator > U256::ZERO);
            let result = MathOps::div(prod0, denominator);
            return result;
        }

        // Make sure the result is less than 2**256.
        // Also prevents denominator == 0
        assert!(denominator > prod1);

        ///////////////////////////////////////////////
        // 512 by 256 division.
        ///////////////////////////////////////////////

        // Make division exact by subtracting the remainder from [prod1 prod0]
        // Compute remainder using mulmod
        let remainder = MathOps::mulmod(a, b, denominator);
        // Subtract 256 bit number from 512 bit number
        prod1 = MathOps::sub(prod1, MathOps::gt(remainder, prod0));
        prod0 = MathOps::sub(prod0, remainder);

        // Factor powers of two out of denominator
        // Compute largest power of two divisor of denominator.
        // Always >= 1.
        let mut twos = (!denominator + 1) & denominator; // denominator must be greater than 0

        // Divide denominator by power of two
        denominator = MathOps::div(denominator, twos);

        // Divide [prod1 prod0] by the factors of two
        prod0 = MathOps::div(prod0, twos);
        // Shift in bits from prod1 into prod0. For this we need
        // to flip `twos` such that it is 2**256 / twos.
        // If twos is zero, then it becomes one

        twos = MathOps::add(
            MathOps::div(MathOps::sub(U256::ZERO, twos), twos),
            U256::ONE,
        );
        prod0 |= MathOps::mul(prod1, twos);

        // Invert denominator mod 2**256
        // Now that denominator is an odd number, it has an inverse
        // modulo 2**256 such that denominator * inv = 1 mod 2**256.
        // Compute the inverse by starting with a seed that is correct
        // correct for four bits. That is, denominator * inv = 1 mod 2**4
        let mut inv = (MathOps::mul(U256::new(3), denominator)) ^ U256::new(2);
        // Now use Newton-Raphson iteration to improve the precision.
        // Thanks to Hensel's lifting lemma, this also works in modular
        // arithmetic, doubling the correct bits in each step.
        inv = MathOps::mul(
            inv,
            MathOps::sub(U256::new(2), MathOps::mul(denominator, inv)),
        ); // inverse mod 2**8
        inv = MathOps::mul(
            inv,
            MathOps::sub(U256::new(2), MathOps::mul(denominator, inv)),
        ); // inverse mod 2**16
        inv = MathOps::mul(
            inv,
            MathOps::sub(U256::new(2), MathOps::mul(denominator, inv)),
        ); // inverse mod 2**32
        inv = MathOps::mul(
            inv,
            MathOps::sub(U256::new(2), MathOps::mul(denominator, inv)),
        ); // inverse mod 2**64
        inv = MathOps::mul(
            inv,
            MathOps::sub(U256::new(2), MathOps::mul(denominator, inv)),
        ); // inverse mod 2**128
        inv = MathOps::mul(
            inv,
            MathOps::sub(U256::new(2), MathOps::mul(denominator, inv)),
        ); // inverse mod 2**256

        // Because the division is now exact we can divide by multiplying
        // with the modular inverse of denominator. This will give us the
        // correct result modulo 2**256. Since the precoditions guarantee
        // that the outcome is less than 2**256, this is the final result.
        // We don't need to compute the high bits of the result and prod1
        // is no longer required.
        let result = MathOps::mul(prod0, inv);
        return result;
    }

    // @notice Calculates ceil(a×b÷denominator) with full precision. Throws if result overflows a uint256 or denominator == 0
    // @param a The multiplicand
    // @param b The multiplier
    // @param denominator The divisor
    // @return result The 256-bit result
    fn mul_div_rounding_up(a: U256, b: U256, denominator: U256) -> U256 {
        let result = FullMath::mul_div(a, b, denominator);
        if MathOps::mulmod(a, b, denominator) > U256::ZERO {
            assert!(result < U256::MAX);
            result + 1
        } else {
            result
        }
    }

    fn unsafe_div_rounding_up(x: U256, y: U256) -> U256 {
        let z = MathOps::add(
            MathOps::div(x, y),
            MathOps::gt(MathOps::modulo(x, y), U256::ZERO),
        );
        z
    }
}

///// For testing

struct FullMathTestEngine;

pub trait FullMathEchidnaTest {
    fn check_mul_div_rounding(x: U256, y: U256, d: U256);
    fn check_mul_div(x: U256, y: U256, d: U256);
    fn check_mul_div_rounding_up(x: U256, y: U256, d: U256);
    fn unsafe_div_round_up(x: U256, y: U256) -> U256;
}

impl FullMathEchidnaTest for FullMathTestEngine {
    fn check_mul_div_rounding(x: U256, y: U256, d: U256) {
        assert!(d > 0);

        let ceiled = FullMath::mul_div_rounding_up(x, y, d);
        let floored = FullMath::mul_div(x, y, d);

        if x.checked_mul(y).unwrap() % d > 0 {
            assert_eq!(ceiled - floored, U256::ONE);
        } else {
            assert_eq!(ceiled, floored);
        }
    }

    fn check_mul_div(x: U256, y: U256, d: U256) {
        assert!(d > 0);
        let z = FullMath::mul_div(x, y, d);
        if x == U256::ZERO || y == U256::ZERO {
            assert_eq!(z, U256::ZERO);
            return;
        }

        // recompute x and y via mul_div of the result of floor(x*y/d), should always be less than original inputs by < d
        let x2 = FullMath::mul_div(z, d, y);
        let y2 = FullMath::mul_div(z, d, x);
        assert!(x2 <= x);
        assert!(y2 <= y);

        assert!(x.checked_sub(x2).unwrap() < d);
        assert!(y.checked_sub(y2).unwrap() < d);
    }

    fn check_mul_div_rounding_up(x: U256, y: U256, d: U256) {
        assert!(d > 0);
        let z = FullMath::mul_div_rounding_up(x, y, d);
        if x == U256::ZERO || y == U256::ZERO {
            assert_eq!(z, U256::ZERO);
            return;
        }

        // recompute x and y via mul_div of the result of floor(x*y/d), should always be less than original inputs by < d
        let x2 = FullMath::mul_div(z, d, y);
        let y2 = FullMath::mul_div(z, d, x);
        assert!(x2 >= x);
        assert!(y2 >= y);

        assert!(x2.checked_sub(x).unwrap() < d);
        assert!(y2.checked_sub(y).unwrap() < d);
    }

    /// @notice Returns ceil(x / y)
    /// @dev division by 0 has unspecified behavior, and must be checked externally
    /// @param x The dividend
    /// @param y The divisor
    /// @return z The quotient, ceil(x / y)
    fn unsafe_div_round_up(x: U256, y: U256) -> U256 {
        let z = MathOps::add(
            MathOps::div(x, y),
            MathOps::gt(MathOps::modulo(x, y), U256::ZERO),
        );
        z
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::full_math::MathOps;
    use std::str::FromStr;

    #[test]
    fn test_mulmod() {
        assert_eq!(
            MathOps::mulmod(
                U256::new(314159265),
                U256::new(314159265),
                U256::new(314159265)
            ),
            0
        );
        assert_eq!(
            MathOps::mulmod(
                U256::new(3141592653589793232),
                U256::new(2718281828459045233),
                U256::new(987234987230498234234)
            ),
            878515112163912297716
        );
        assert_eq!(
            MathOps::mulmod(
                U256::new(123456789),
                U256::new(101112131415),
                U256::new(981238791623981726391827)
            ),
            12482979073441926435
        );
        assert_eq!(
            MathOps::mulmod(
                U256::from_str(
                    "11579208923731619542357098500868790785326998466564056403945758400791312963993"
                )
                .unwrap(),
                U256::from_str(
                    "57896044618658097711785492504343953926634992332820282019728792003956564819968"
                )
                .unwrap(),
                U256::from_str(
                    "7257920892373161954235709850086879078532699846656405640394575840079131296399"
                )
                .unwrap()
            ),
            U256::from_str(
                "6027955390367321265543803621229480096361388853679971715171647694262792444634"
            )
            .unwrap()
        );
    }

    use crate::fixed_point_128;
    use std::panic;

    #[test]
    fn test_mul_div() {
        // reverts if denominator is 0
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div(fixed_point_128::get_q128(), U256::new(5), U256::new(0));
        })
        .is_err());
        // reverts if denominator is 0 and numerator overflows
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div(
                fixed_point_128::get_q128(),
                fixed_point_128::get_q128(),
                U256::new(0),
            );
        })
        .is_err());
        assert_eq!(
            FullMath::mul_div(
                fixed_point_128::get_q128(),
                fixed_point_128::get_q128(),
                U256::MAX
            ),
            U256::new(1)
        );
        // overflow
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div(U256::MAX, U256::MAX, U256::MAX - 1);
        })
        .is_err());
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div(U256::MAX, U256::MAX, U256::MAX - 1);
        })
        .is_err());
        // all max inputs
        assert_eq!(
            FullMath::mul_div(U256::MAX, U256::MAX, U256::MAX),
            U256::MAX
        );

        // accurate without phantom overflow
        {
            let x = fixed_point_128::get_q128();
            let y = FullMath::mul_div(U256::new(50), fixed_point_128::get_q128(), U256::new(100));
            let modulo =
                FullMath::mul_div(U256::new(150), fixed_point_128::get_q128(), U256::new(100));
            let answer = fixed_point_128::get_q128() / U256::new(3);
            assert_eq!(FullMath::mul_div(x, y, modulo), answer);
        }

        // accurate with phantom overflow
        {
            let x = fixed_point_128::get_q128();
            let y = x * U256::new(35);
            let modulo = x * U256::new(8);
            let answer = U256::new(4375) * fixed_point_128::get_q128() / U256::new(1000);
            assert_eq!(FullMath::mul_div(x, y, modulo), answer);
        }
        // accurate with phantom overflow and repeating decimal
        {
            let x = fixed_point_128::get_q128();
            let y = x * U256::new(1000);
            let modulo = x * U256::new(3000);
            let answer = U256::new(1) * fixed_point_128::get_q128() / U256::new(3);
            assert_eq!(FullMath::mul_div(x, y, modulo), answer);
        }
    }

    #[test]
    fn test_mul_div_rounding_up() {
        // reverts if denominator is 0
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div_rounding_up(fixed_point_128::get_q128(), U256::new(5), U256::new(0));
        })
        .is_err());
        // reverts if denominator is 0 and numerator overflows
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div_rounding_up(
                fixed_point_128::get_q128(),
                fixed_point_128::get_q128(),
                U256::new(0),
            );
        })
        .is_err());
        // reverts if output overflows uint256
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div_rounding_up(
                fixed_point_128::get_q128(),
                fixed_point_128::get_q128(),
                U256::new(1),
            );
        })
        .is_err());
        // reverts on overflow with all max inputs
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div_rounding_up(U256::MAX, U256::MAX, U256::MAX.sub(U256::ONE));
        })
        .is_err());
        // reverts if mulDiv overflows 256 bits after rounding up
        assert!(panic::catch_unwind(|| {
            FullMath::mul_div_rounding_up(
                U256::new(535006138814359),
                U256::from_str("432862656469423142931042426214547535783388063929571229938474969")
                    .unwrap()
                    .as_u256(),
                U256::new(2),
            );
        })
        .is_err());
        // reverts if mulDiv overflows 256 bits after rounding up case 2
        assert!(panic::catch_unwind(|| {
      FullMath::mul_div_rounding_up(U256::from_str("115792089237316195423570985008687907853269984659341747863450311749907997002549").unwrap().as_u256(), U256::from_str("115792089237316195423570985008687907853269984659341747863450311749907997002550").unwrap().as_u256(), U256::from_str("115792089237316195423570985008687907853269984653042931687443039491902864365164").unwrap().as_u256());
    }).is_err());
        // all max inputs
        {
            let x = U256::MAX;
            let y = x;
            let modulo = x;
            let answer = x;
            assert_eq!(FullMath::mul_div_rounding_up(x, y, modulo), answer);
        }
        // accurate without phantom overflow
        {
            let x = fixed_point_128::get_q128();
            let y = FullMath::mul_div(U256::new(50), fixed_point_128::get_q128(), U256::new(100));
            let modulo =
                FullMath::mul_div(U256::new(150), fixed_point_128::get_q128(), U256::new(100));
            let answer = fixed_point_128::get_q128() / U256::new(3) + U256::ONE;
            assert_eq!(FullMath::mul_div_rounding_up(x, y, modulo), answer);
        }
        // accurate with phantom overflow
        {
            let x = fixed_point_128::get_q128();
            let y = x * U256::new(35);
            let modulo = x * U256::new(8);
            let answer = U256::new(4375) * fixed_point_128::get_q128() / U256::new(1000);
            assert_eq!(FullMath::mul_div_rounding_up(x, y, modulo), answer);
        }
        // accurate with phantom overflow and repeating decimal
        {
            let x = fixed_point_128::get_q128();
            let y = x * U256::new(1000);
            let modulo = x * U256::new(3000);
            let answer = U256::new(1) * fixed_point_128::get_q128() / U256::new(3) + U256::ONE;
            assert_eq!(FullMath::mul_div_rounding_up(x, y, modulo), answer);
        }
    }

    fn pseudo_random_big_number(seed: U256) -> U256 {
        let mut res = U256::ZERO;
        let mut tmp = seed;
        for i in 0..100 {
            res = res.overflowing_add(tmp).0;
            tmp = tmp.overflowing_mul(tmp.overflowing_add(U256::new(i)).0).0;
        }
        tmp
    }

    #[test]
    fn check_random_inputs() {
        let mut tests = Vec::new();

        for i in 2..3 {
            let x = pseudo_random_big_number(U256::new(i) + pseudo_random_big_number(U256::ONE));
            let y = pseudo_random_big_number(U256::new(i) + pseudo_random_big_number(U256::new(2)));
            let d = pseudo_random_big_number(U256::new(i) + pseudo_random_big_number(U256::new(3)));
            tests.push((x, y, d));
        }

        for (x, y, d) in tests {
            if d <= U256::ZERO {
                assert!(panic::catch_unwind(|| {
                    FullMath::mul_div(x, y, d);
                })
                .is_err());
                assert!(panic::catch_unwind(|| {
                    FullMath::mul_div_rounding_up(x, y, d);
                })
                .is_err());
            } else if x == U256::ZERO || y == U256::ZERO {
                assert_eq!(FullMath::mul_div(x, y, d), U256::ZERO);
                assert_eq!(FullMath::mul_div_rounding_up(x, y, d), U256::ZERO);
                // } else if x * y / d > U256::max_value() {
                //   expect(full_math_mul_div(x, y, d).is_err());
                //   expect(full_math_mul_div_rounding_up(x, y, d).is_err());
            } else {
                let result = panic::catch_unwind(|| {
                    let floored = FullMath::mul_div(x, y, d);
                    let ceiled = FullMath::mul_div_rounding_up(x, y, d);

                    let remainder = MathOps::mulmod(x, y, d);

                    if remainder > U256::ZERO {
                        floored + U256::ONE == ceiled
                    } else {
                        floored == ceiled
                    }
                });

                match result {
                    Ok(diff_less_than_or_equal_to_1) => {
                        assert_eq!(diff_less_than_or_equal_to_1, true);
                    }
                    Err(_) => {}
                }
            }
        }
    }
}
