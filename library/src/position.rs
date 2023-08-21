use crate::num256::U256;
use crate::{fixed_point_128, liquidity_math};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use super::full_math::FullMath;
use crate::full_math::FullMathTrait;

// info stored for each user's position
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default)]
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

impl PositionInfo {
    /// Credits accumulated fees to a user's position
    pub fn update(
        &mut self,
        liquidity_delta: i128,
        fee_growth_inside_0_x128: U256,
        fee_growth_inside_1_x128: U256,
    ) {
        let liquidity_next: u128 = if liquidity_delta == 0 && self.liquidity > 0 {
            self.liquidity
        } else {
            liquidity_math::add_delta(self.liquidity, liquidity_delta)
        };

        // calculate accumulated fees
        let tokens_owed_0 = FullMath::mul_div(
            fee_growth_inside_0_x128 - self.fee_growth_inside_0_last_x128,
            U256::from(self.liquidity),
            fixed_point_128::get_q128(),
        )
        .as_u128();

        let tokens_owed_1 = FullMath::mul_div(
            fee_growth_inside_1_x128 - self.fee_growth_inside_1_last_x128,
            U256::from(self.liquidity),
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
    fn test_update() {
        // TODO: @galin-chung-nguyen
    }
}
