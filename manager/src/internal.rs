use std::cmp::Ordering;

use near_sdk::{env, AccountId};
use zswap_math_library::num256::U256;
use zswap_math_library::{liquidity_math, pool_account, tick_math};

use crate::error::TOKENS_MUST_BE_DIFFERENT;
use crate::pool::Slot0;
use crate::Contract;

impl Contract {
    pub fn internal_get_pool(
        &self,
        token_0: &AccountId,
        token_1: &AccountId,
        fee: u32,
    ) -> AccountId {
        let ordered_token_0;
        let ordered_token_1;
        match token_0.cmp(token_1) {
            Ordering::Less => {
                ordered_token_0 = token_0;
                ordered_token_1 = token_1;
            }
            Ordering::Greater => {
                ordered_token_0 = token_1;
                ordered_token_1 = token_0;
            }
            Ordering::Equal => env::panic_str(TOKENS_MUST_BE_DIFFERENT),
        }

        pool_account::compute_account(
            self.factory.clone(),
            ordered_token_0.clone(),
            ordered_token_1.clone(),
            fee,
        )
    }

    pub fn internal_calculate_liquidity(
        &self,
        slot_0: Slot0,
        lower_tick: i32,
        upper_tick: i32,
        amount_0: u128,
        amount_1: u128,
    ) -> u128 {
        let sqrt_price_x96 = slot_0.sqrt_price_x96;
        let sqrt_price_lower_x96 = tick_math::get_sqrt_ratio_at_tick(lower_tick);
        let sqrt_price_upper_x96 = tick_math::get_sqrt_ratio_at_tick(upper_tick);
        liquidity_math::get_liquidity_for_amounts(
            U256::from(sqrt_price_x96.0),
            sqrt_price_lower_x96,
            sqrt_price_upper_x96,
            amount_0,
            amount_1,
        )
    }
}
