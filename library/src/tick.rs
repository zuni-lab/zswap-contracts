use ethnum::AsI256;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::full_math::MathOps;
use crate::liquidity_math;
use crate::num160::U160;
use crate::num24::{AsI24, AsU24, I24};
use crate::num256::U256;
use crate::num56::{I56, U56};
use crate::tick_math::TickConstants;

// info stored for each initialized individual tick
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default)]
pub struct TickInfo {
    // the total position liquidity that references this tick
    pub liquidity_gross: u128,
    // amount of net liquidity added (subtracted) when tick is crossed from left to right (right to left),
    pub liquidity_net: i128,
    // fee growth per unit of liquidity on the _other_ side of this tick (relative to the current tick)
    // only has relative meaning, not absolute — the value depends on when the tick is initialized
    fee_growth_outside_0_x128: U256,
    fee_growth_outside_1_x128: U256,
    // the cumulative tick value on the other side of the tick
    pub tick_cumulative_outside: I56,
    // the seconds per unit of liquidity on the _other_ side of this tick (relative to the current tick)
    // only has relative meaning, not absolute — the value depends on when the tick is initialized
    pub seconds_per_liquidity_outside_x128: U160,
    // the seconds spent on the other side of the tick (relative to the current tick)
    // only has relative meaning, not absolute — the value depends on when the tick is initialized
    pub seconds_outside: u32,
    // true iff the tick is initialized, i.e. the value is exactly equivalent to the expression liquidityGross != 0
    // these 8 bits are set to prevent fresh stores when crossing newly initialized ticks
    pub initialized: bool,
}

impl TickInfo {
    /// Updates a tick and returns true if the tick was flipped from initialized to uninitialized, or vice versa
    ///
    /// Return flipped whether the tick was flipped from initialized to uninitialized, or vice versa
    pub fn update(
        &mut self,
        tick: I24,
        current_tick: I24,
        liquidity_delta: i128,
        fee_growth_global_0_x128: U256,
        fee_growth_global_1_x128: U256,
        upper: bool,
    ) -> bool {
        let liquidity_gross_before = self.liquidity_gross;
        let liquidity_gross_after =
            liquidity_math::add_delta(liquidity_gross_before, liquidity_delta);

        // assert!(liquidity_gross_after <= max_liquidity, "LO");

        let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

        if liquidity_gross_before == 0 {
            // by convention, we assume that all growth before a tick was initialized happened _below_ the tick
            if tick <= current_tick {
                self.fee_growth_outside_0_x128 = fee_growth_global_0_x128;
                self.fee_growth_outside_1_x128 = fee_growth_global_1_x128;
                // self.seconds_per_liquidity_outside_x128 = seconds_per_liquidity_cumulative_x128;
                // self.tick_cumulative_outside = tick_cumulative;
                // self.seconds_outside = time;
            }
            self.initialized = true;
        }

        self.liquidity_gross = liquidity_gross_after;

        // when the lower (upper) tick is crossed left to right (right to left), liquidity must be added (removed)
        self.liquidity_net = if upper {
            self.liquidity_net
                .as_i256()
                .checked_sub(liquidity_delta.as_i256())
                .unwrap()
                .as_i128()
        } else {
            self.liquidity_net
                .as_i256()
                .checked_add(liquidity_delta.as_i256())
                .unwrap()
                .as_i128()
        };

        flipped
    }
}

/// @title Tick
/// @notice Contains functions for managing tick processes and relevant calculations

/// @notice Derives max liquidity per tick from given tick spacing
/// @dev Executed within the pool constructor
/// @param tickSpacing The amount of required tick separation, realized in multiples of `tickSpacing`
///     e.g., a tickSpacing of 3 requires ticks to be initialized every 3rd tick i.e., ..., -6, -3, 0, 3, 6, ...
/// @return The max liquidity per tick
pub fn tick_spacing_to_max_liquidity_per_tick(tick_spacing: I24) -> u128 {
    let min_tick = ((TickConstants::MIN_TICK.as_i256() / tick_spacing.as_i256())
        * tick_spacing.as_i256())
    .as_i24();
    let max_tick = ((TickConstants::MAX_TICK.as_i256() / tick_spacing.as_i256())
        * tick_spacing.as_i256())
    .as_i24();
    let num_ticks = U256::from((max_tick - min_tick) / tick_spacing + 1).as_u24();
    u128::MAX / (num_ticks as u128)
}

/// Retrieves fee growth data
///
/// Return `fee_growth_inside_0_x128` & `fee_growth_inside_1_x128`
/// The all-time fee growth in token_0 and token_1,
/// per unit of liquidity, inside the position's tick boundaries
pub fn get_fee_growth_inside(
    lower_tick: I24,
    upper_tick: I24,
    lower: &TickInfo,
    upper: &TickInfo,
    current_tick: I24,
    fee_growth_global_0_x128: U256,
    fee_growth_global_1_x128: U256,
) -> [U256; 2] {
    // calculate fee growth below
    let fee_growth_below0_x128: U256;
    let fee_growth_below1_x128: U256;
    if current_tick >= lower_tick {
        fee_growth_below0_x128 = lower.fee_growth_outside_0_x128;
        fee_growth_below1_x128 = lower.fee_growth_outside_1_x128;
    } else {
        fee_growth_below0_x128 =
            MathOps::sub(fee_growth_global_0_x128, lower.fee_growth_outside_0_x128);
        fee_growth_below1_x128 =
            MathOps::sub(fee_growth_global_1_x128, lower.fee_growth_outside_1_x128);
    }

    // Calculate fee growth above
    let fee_growth_above0_x128: U256;
    let fee_growth_above1_x128: U256;
    if current_tick < upper_tick {
        fee_growth_above0_x128 = upper.fee_growth_outside_0_x128;
        fee_growth_above1_x128 = upper.fee_growth_outside_1_x128;
    } else {
        fee_growth_above0_x128 =
            MathOps::sub(fee_growth_global_0_x128, upper.fee_growth_outside_0_x128);
        fee_growth_above1_x128 =
            MathOps::sub(fee_growth_global_1_x128, upper.fee_growth_outside_1_x128);
    }

    let fee_growth_inside_0_x128 = MathOps::sub(
        MathOps::sub(fee_growth_global_0_x128, fee_growth_below0_x128),
        fee_growth_above0_x128,
    );
    let fee_growth_inside_1_x128 = MathOps::sub(
        MathOps::sub(fee_growth_global_1_x128, fee_growth_below1_x128),
        fee_growth_above1_x128,
    );

    [fee_growth_inside_0_x128, fee_growth_inside_1_x128]
}

/// !MODIFIED
/// @notice Transitions to next tick as needed by price movement
/// @param self The mapping containing all tick information for initialized ticks
/// @param tick The destination tick of the transition
/// @param feeGrowthGlobal0X128 The all-time global fee growth, per unit of liquidity, in token0
/// @param feeGrowthGlobal1X128 The all-time global fee growth, per unit of liquidity, in token1
/// @param secondsPerLiquidityCumulativeX128 The current seconds per liquidity
/// @param tickCumulative The tick * time elapsed since the pool was first initialized
/// @param time The current block.timestamp
/// @return liquidityNet The amount of liquidity added (subtracted) when tick is crossed from left to right (right to left)
pub fn cross(
    info: &mut TickInfo,
    fee_growth_global_0_x128: U56,
    fee_growth_global_1_x128: U56,
    seconds_per_liquidity_cumulative_x128: U160,
    tick_cumulative: I56,
    time: u32,
) -> i128 {
    info.fee_growth_outside_0_x128 =
        U256::from(fee_growth_global_0_x128) - info.fee_growth_outside_0_x128;
    info.fee_growth_outside_1_x128 =
        U256::from(fee_growth_global_1_x128) - info.fee_growth_outside_1_x128;
    info.seconds_per_liquidity_outside_x128 =
        seconds_per_liquidity_cumulative_x128 - info.seconds_per_liquidity_outside_x128;
    info.tick_cumulative_outside = tick_cumulative - info.tick_cumulative_outside;
    info.seconds_outside = time - info.seconds_outside;
    info.liquidity_net
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    use ethnum::AsI256;
    use std::collections::HashMap;

    use crate::num160::U160;
    use crate::num24::AsI24;
    use crate::tick;
    use crate::tick::{cross, get_fee_growth_inside, TickInfo};

    const LOW: u128 = 500;
    const MEDIUM: u128 = 3000;
    const HIGH: u128 = 10000;

    fn create_tick_spacings() -> HashMap<u128, u128> {
        let mut map = HashMap::new();
        map.insert(LOW, 10);
        map.insert(MEDIUM, 60);
        map.insert(HIGH, 200);
        map
    }

    #[test]
    fn test_tick_spacing_to_max_liquidity_per_tick() {
        let tick_spacings = create_tick_spacings();
        let expected_values: [(u128, u128); 5] = [
            (
                *tick_spacings.get(&LOW).unwrap(),
                1917569901783203986719870431555990,
            ),
            (
                *tick_spacings.get(&MEDIUM).unwrap(),
                11505743598341114571880798222544994,
            ),
            (
                *tick_spacings.get(&HIGH).unwrap(),
                38350317471085141830651933667504588,
            ),
            (887272, u128::MAX / 3),
            (2302, 441351967472034323558203122479595605),
        ];

        for (tick_spacing, expected) in expected_values.iter() {
            let max_liquidity_per_tick =
                tick::tick_spacing_to_max_liquidity_per_tick(tick_spacing.as_i256().as_i24());
            let expected_liquidity: u128 = (*expected).into();
            assert_eq!(max_liquidity_per_tick, expected_liquidity);
        }
    }

    #[test]
    fn test_fee_growth_inside_uninitialized_ticks_above() {
        {
            // returns all for two uninitialized ticks if tick is inside
            let [fee_growth_inside_0_x128, fee_growth_inside_1_x128] = get_fee_growth_inside(
                -2,
                2,
                &TickInfo::default(),
                &TickInfo::default(),
                0,
                U256::from(15),
                U256::from(15),
            );
            assert_eq!(fee_growth_inside_0_x128, U256::from(15));
            assert_eq!(fee_growth_inside_1_x128, U256::from(15));
        }
        {
            // returns 0 for two uninitialized ticks if tick is above
            let [fee_growth_inside_0_x128, fee_growth_inside_1_x128] = get_fee_growth_inside(
                -2,
                2,
                &TickInfo::default(),
                &TickInfo::default(),
                4,
                U256::from(15),
                U256::from(15),
            );
            assert_eq!(fee_growth_inside_0_x128, U256::zero());
            assert_eq!(fee_growth_inside_1_x128, U256::zero());
        }
        {
            // returns 0 for two uninitialized ticks if tick is below
            let [fee_growth_inside_0_x128, fee_growth_inside_1_x128] = get_fee_growth_inside(
                -2,
                2,
                &TickInfo::default(),
                &TickInfo::default(),
                -4,
                U256::from(15),
                U256::from(15),
            );
            assert_eq!(fee_growth_inside_0_x128, U256::zero());
            assert_eq!(fee_growth_inside_1_x128, U256::zero());
        }
        {
            // subtracts upper tick if below
            let lower = TickInfo::default();
            let upper = TickInfo {
                fee_growth_outside_0_x128: U256::from(2),
                fee_growth_outside_1_x128: U256::from(3),
                liquidity_gross: 0,
                liquidity_net: 0,
                seconds_per_liquidity_outside_x128: U160::zero(),
                tick_cumulative_outside: 0,
                seconds_outside: 0,
                initialized: true,
            };

            let [fee_growth_inside_0_x128, fee_growth_inside_1_x128] =
                get_fee_growth_inside(-2, 2, &lower, &upper, 0, U256::from(15), U256::from(15));
            assert_eq!(fee_growth_inside_0_x128, U256::from(13));
            assert_eq!(fee_growth_inside_1_x128, U256::from(12));
        }
        {
            // subtracts upper tick if above
            let upper = TickInfo::default();
            let lower = TickInfo {
                fee_growth_outside_0_x128: U256::from(2),
                fee_growth_outside_1_x128: U256::from(3),
                liquidity_gross: 0,
                liquidity_net: 0,
                seconds_per_liquidity_outside_x128: U160::zero(),
                tick_cumulative_outside: 0,
                seconds_outside: 0,
                initialized: true,
            };

            let [fee_growth_inside_0_x128, fee_growth_inside_1_x128] =
                get_fee_growth_inside(-2, 2, &lower, &upper, 0, U256::from(15), U256::from(15));
            assert_eq!(fee_growth_inside_0_x128, U256::from(13));
            assert_eq!(fee_growth_inside_1_x128, U256::from(12));
        }
        {
            // subtracts upper and lower tick if inside
            let lower = TickInfo {
                fee_growth_outside_0_x128: U256::from(2),
                fee_growth_outside_1_x128: U256::from(3),
                liquidity_gross: 0,
                liquidity_net: 0,
                seconds_per_liquidity_outside_x128: U160::zero(),
                tick_cumulative_outside: 0,
                seconds_outside: 0,
                initialized: true,
            };
            let upper = TickInfo {
                fee_growth_outside_0_x128: U256::from(4),
                fee_growth_outside_1_x128: U256::from(1),
                liquidity_gross: 0,
                liquidity_net: 0,
                seconds_per_liquidity_outside_x128: U160::zero(),
                tick_cumulative_outside: 0,
                seconds_outside: 0,
                initialized: true,
            };

            let [fee_growth_inside_0_x128, fee_growth_inside_1_x128] =
                get_fee_growth_inside(-2, 2, &lower, &upper, 0, U256::from(15), U256::from(15));
            assert_eq!(fee_growth_inside_0_x128, U256::from(9));
            assert_eq!(fee_growth_inside_1_x128, U256::from(11));
        }
        {
            // works correctly with overflow on inside tick
            let lower = TickInfo {
                fee_growth_outside_0_x128: U256::MAX - U256::from(3),
                fee_growth_outside_1_x128: U256::MAX - U256::from(2),
                liquidity_gross: 0,
                liquidity_net: 0,
                seconds_per_liquidity_outside_x128: U160::zero(),
                tick_cumulative_outside: 0,
                seconds_outside: 0,
                initialized: true,
            };
            let upper = TickInfo {
                fee_growth_outside_0_x128: U256::from(3),
                fee_growth_outside_1_x128: U256::from(5),
                liquidity_gross: 0,
                liquidity_net: 0,
                seconds_per_liquidity_outside_x128: U160::zero(),
                tick_cumulative_outside: 0,
                seconds_outside: 0,
                initialized: true,
            };

            let [fee_growth_inside_0_x128, fee_growth_inside_1_x128] =
                get_fee_growth_inside(-2, 2, &lower, &upper, 0, U256::from(15), U256::from(15));
            assert_eq!(fee_growth_inside_0_x128, U256::from(16));
            assert_eq!(fee_growth_inside_1_x128, U256::from(13));
        }
    }

    #[test]
    fn test_cross() {
        {
            // flips the growth variables
            let mut tick = TickInfo {
                fee_growth_outside_0_x128: U256::from(1),
                fee_growth_outside_1_x128: U256::from(2),
                liquidity_gross: 3,
                liquidity_net: 4,
                seconds_per_liquidity_outside_x128: U160::from(5),
                tick_cumulative_outside: 6,
                seconds_outside: 7,
                initialized: true,
            };

            cross(&mut tick, 7, 9, U160::from(8), 15, 10);

            assert_eq!(tick.fee_growth_outside_0_x128, U256::from(6));
            assert_eq!(tick.fee_growth_outside_1_x128, U256::from(7));
            assert_eq!(tick.seconds_per_liquidity_outside_x128, U256::from(3));
            assert_eq!(tick.tick_cumulative_outside, 9);
            assert_eq!(tick.seconds_outside, 3);
        }
        {
            // two flips are no op
            let mut tick = TickInfo {
                fee_growth_outside_0_x128: U256::from(1),
                fee_growth_outside_1_x128: U256::from(2),
                liquidity_gross: 3,
                liquidity_net: 4,
                seconds_per_liquidity_outside_x128: U160::from(5),
                tick_cumulative_outside: 6,
                seconds_outside: 7,
                initialized: true,
            };

            cross(&mut tick, 7, 9, U160::from(8), 15, 10);
            cross(&mut tick, 7, 9, U160::from(8), 15, 10);

            assert_eq!(tick.fee_growth_outside_0_x128, U256::from(1));
            assert_eq!(tick.fee_growth_outside_1_x128, U256::from(2));
            assert_eq!(tick.seconds_per_liquidity_outside_x128, U256::from(5));
            assert_eq!(tick.tick_cumulative_outside, 6);
            assert_eq!(tick.seconds_outside, 7);
        }
    }

    #[test]
    fn test_update() {
        // TODO: @galin-chung-nguyen
    }

    #[test]
    fn test_clear() {
        // TODO: @galin-chung-nguyen
    }
}
