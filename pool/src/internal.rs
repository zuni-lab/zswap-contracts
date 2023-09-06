use ethnum::I256;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{env, AccountId, CryptoHash, Promise};
use zswap_math_library::{
    liquidity_math, num256::U256, sqrt_price_math, tick, tick_bitmap::flip_tick, tick_math,
};

use crate::{
    error::{INSUFFICIENT_INPUT_AMOUNT, NOT_AUTHORIZED},
    Contract,
};

impl Contract {
    pub fn get_balance_0_promise(&self) -> Promise {
        ext_ft_core::ext(self.token_0.clone()).ft_balance_of(env::current_account_id())
    }

    pub fn get_balance_1_promise(&self) -> Promise {
        ext_ft_core::ext(self.token_1.clone()).ft_balance_of(env::current_account_id())
    }

    pub fn get_position_key(
        &self,
        owner: &AccountId,
        lower_tick: i32,
        upper_tick: i32,
    ) -> CryptoHash {
        env::keccak256_array(
            [
                owner.as_bytes(),
                &lower_tick.to_le_bytes(),
                &upper_tick.to_le_bytes(),
            ]
            .concat()
            .as_slice(),
        )
    }

    pub fn modify_position(
        &mut self,
        owner: &AccountId,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: i128,
    ) -> [i128; 2] {
        let position_key = self.get_position_key(owner, lower_tick, upper_tick);
        let mut position = self.positions.get(&position_key).unwrap_or_default();

        let current_tick = self.slot_0.tick;
        let fee_growth_global_0_x128 = self.fee_growth_global_0_x128;
        let fee_growth_global_1_x128 = self.fee_growth_global_1_x128;

        let mut lower_tick_info = self.ticks.get(&lower_tick).unwrap_or_default();
        let flipped_lower = lower_tick_info.update(
            lower_tick,
            current_tick,
            liquidity_delta,
            fee_growth_global_0_x128,
            fee_growth_global_1_x128,
            false,
        );
        self.ticks.insert(&lower_tick, &lower_tick_info);

        let mut upper_tick_info = self.ticks.get(&upper_tick).unwrap_or_default();
        let flipped_upper = upper_tick_info.update(
            upper_tick,
            current_tick,
            liquidity_delta,
            fee_growth_global_0_x128,
            fee_growth_global_1_x128,
            true,
        );
        self.ticks.insert(&upper_tick, &upper_tick_info);

        if flipped_lower {
            flip_tick(&mut self.tick_bitmap, lower_tick, self.tick_spacing as i32);
        }

        if flipped_upper {
            flip_tick(&mut self.tick_bitmap, upper_tick, self.tick_spacing as i32);
        }

        let fees_growth_inside_x128 = tick::get_fee_growth_inside(
            lower_tick,
            upper_tick,
            &lower_tick_info,
            &upper_tick_info,
            current_tick,
            fee_growth_global_0_x128,
            fee_growth_global_1_x128,
        );

        position.update(
            liquidity_delta,
            fees_growth_inside_x128[0],
            fees_growth_inside_x128[1],
        );
        self.positions.insert(&position_key, &position);

        let sqrt_current_price = self.slot_0.sqrt_price_x96.0;
        let mut amount_0 = I256::ZERO;
        let mut amount_1 = I256::ZERO;
        if current_tick < lower_tick {
            amount_0 = sqrt_price_math::get_amount_0_delta_signed(
                tick_math::get_sqrt_ratio_at_tick(lower_tick),
                tick_math::get_sqrt_ratio_at_tick(upper_tick),
                liquidity_delta,
            );
        } else if current_tick < upper_tick {
            amount_0 = sqrt_price_math::get_amount_0_delta_signed(
                U256::from(sqrt_current_price),
                tick_math::get_sqrt_ratio_at_tick(upper_tick),
                liquidity_delta,
            );
            amount_1 = sqrt_price_math::get_amount_1_delta_signed(
                tick_math::get_sqrt_ratio_at_tick(lower_tick),
                U256::from(sqrt_current_price),
                liquidity_delta,
            );
            self.liquidity = liquidity_math::add_liquidity(self.liquidity, liquidity_delta);
        } else {
            amount_1 = sqrt_price_math::get_amount_1_delta_signed(
                tick_math::get_sqrt_ratio_at_tick(lower_tick),
                tick_math::get_sqrt_ratio_at_tick(upper_tick),
                liquidity_delta,
            );
        }

        [amount_0.as_i128(), amount_1.as_i128()]
    }

    pub fn internal_collect_token_0_to_mint(
        &mut self,
        owner: &AccountId,
        caller: &AccountId,
        amount: u128,
    ) {
        if owner != caller {
            let approval = self.approved_token_0.get(owner);
            match approval {
                Some(approved) => {
                    if &approved != caller {
                        env::panic_str(NOT_AUTHORIZED);
                    }
                }
                None => {
                    env::panic_str(NOT_AUTHORIZED);
                }
            }
        }

        let deposited_token_opt = self.deposited_token_0.get(owner);
        match deposited_token_opt {
            Some(deposited) => {
                if deposited < amount {
                    env::panic_str(INSUFFICIENT_INPUT_AMOUNT);
                }

                self.deposited_token_0.insert(owner, &(deposited - amount));
            }
            None => {
                env::panic_str(INSUFFICIENT_INPUT_AMOUNT);
            }
        }
    }

    pub fn internal_collect_token_1_to_mint(
        &mut self,
        owner: &AccountId,
        caller: &AccountId,
        amount: u128,
    ) {
        if owner != caller {
            let approval = self.approved_token_1.get(owner);
            match approval {
                Some(approved) => {
                    if &approved != caller {
                        env::panic_str(NOT_AUTHORIZED);
                    }
                }
                None => {
                    env::panic_str(NOT_AUTHORIZED);
                }
            }
        }

        let deposited_token_opt = self.deposited_token_1.get(owner);
        match deposited_token_opt {
            Some(deposited) => {
                if deposited < amount {
                    env::panic_str(INSUFFICIENT_INPUT_AMOUNT);
                }

                self.deposited_token_1.insert(owner, &(deposited - amount));
            }
            None => {
                env::panic_str(INSUFFICIENT_INPUT_AMOUNT);
            }
        }
    }
}
