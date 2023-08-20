use ethnum::{AsU256, U256};

use super::full_math::FullMath;
use crate::full_math::FullMathTrait;

// info stored for each user's position
pub struct PositionInfo {
    // the amount of liquidity owned by this position
    pub liquidity: u128,
    // fee growth per unit of liquidity as of the last update to liquidity or fees owed
    pub fee_growth_inside_0_last_x128: U256,
    pub fee_growth_inside_1_last_x128: U256,
    // the fees owed to the position owner in token0/token1
    pub tokens_owed_0: u128,
    pub tokens_owed_1: u128,
}

use crate::{fixed_point_128, liquidity_math};

// Define trait for updating position
trait UpdatePosition {
    /// @notice Credits accumulated fees to a user's position
    /// @param self The individual position to update
    /// @param liquidityDelta The change in pool liquidity as a result of the position update
    /// @param feeGrowthInside0X128 The all-time fee growth in token0, per unit of liquidity, inside the position's tick boundaries
    /// @param feeGrowthInside1X128 The all-time fee growth in token1, per unit of liquidity, inside the position's tick boundaries
    fn update(
        &mut self,
        liquidity_delta: i128,
        fee_growth_inside_0_x128: U256,
        fee_growth_inside_1_x128: U256,
    );
}

impl UpdatePosition for PositionInfo {
    /// @notice Credits accumulated fees to a user's position
    /// @param self The individual position to update
    /// @param liquidityDelta The change in pool liquidity as a result of the position update
    /// @param feeGrowthInside0X128 The all-time fee growth in token0, per unit of liquidity, inside the position's tick boundaries
    /// @param feeGrowthInside1X128 The all-time fee growth in token1, per unit of liquidity, inside the position's tick boundaries
    fn update(
        &mut self,
        liquidity_delta: i128,
        fee_growth_inside_0_x128: U256,
        fee_growth_inside_1_x128: U256,
    ) {
        let liquidity_next: u128;
        if liquidity_delta == 0 {
            assert!(self.liquidity > 0, "NP"); // disallow pokes for 0 liquidity positions
            liquidity_next = self.liquidity;
        } else {
            liquidity_next = liquidity_math::add_delta(self.liquidity, liquidity_delta);
        }

        // calculate accumulated fees
        let tokens_owed_0 = FullMath::mul_div(
            fee_growth_inside_0_x128 - self.fee_growth_inside_0_last_x128,
            self.liquidity.as_u256(),
            fixed_point_128::get_q128(),
        )
        .as_u128();

        let tokens_owed_1 = FullMath::mul_div(
            fee_growth_inside_1_x128 - self.fee_growth_inside_1_last_x128,
            self.liquidity.as_u256(),
            fixed_point_128::get_q128(),
        )
        .as_u128();

        // update the position
        if liquidity_delta != 0 {
            self.liquidity = liquidity_next;
        }
        self.fee_growth_inside_0_last_x128 = fee_growth_inside_0_x128;
        self.fee_growth_inside_1_last_x128 = fee_growth_inside_1_x128;

        if tokens_owed_0 > 0 || tokens_owed_1 > 0 {
            // overflow is acceptable, have to withdraw before you hit type(uint128).max fees
            self.tokens_owed_0 = self.tokens_owed_0.overflowing_add(tokens_owed_0).0;
            self.tokens_owed_1 = self.tokens_owed_1.overflowing_add(tokens_owed_1).0;
        }
    }
}

#[cfg(test)]
mod tests {
    // use ethnum::U256;
    // use std::panic;

    #[test]
    fn test_position_update() {
        // assert_eq!(Position::add_delta(1, 0), 1);
        // assert_eq!(Position::add_delta(1, -1), 0);
        // assert_eq!(Position::add_delta(1, 1), 2);
        // // 2**128-15 + 15 overflows
        // assert!(panic::catch_unwind(|| {
        //   Position::add_delta((U256::new(2).pow(128) - U256::new(15)).as_u128(), 15);
        // }).is_err());
        // // 0 + -1 underflows
        // assert!(panic::catch_unwind(|| {
        //   Position::add_delta(0, -1);
        // }).is_err());
        // // 3 + -4 underflows
        // assert!(panic::catch_unwind(|| {
        //   Position::add_delta(3, -4);
        // }).is_err());
    }
}
