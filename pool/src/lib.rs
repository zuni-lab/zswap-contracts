use ethnum::I256;
use near_contract_standards::fungible_token::core::ext_ft_core;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, AccountId, BorshStorageKey, CryptoHash, PanicOnDefault, PromiseOrValue,
    ONE_YOCTO,
};

use zswap_math_library::full_math::{FullMath, FullMathTrait};
use zswap_math_library::num160::AsU160;
use zswap_math_library::num256::U256;
use zswap_math_library::position::PositionInfo;
use zswap_math_library::tick::TickInfo;
use zswap_math_library::tick_math::TickConstants;
use zswap_math_library::{fixed_point_128, liquidity_math, swap_math, tick_bitmap, tick_math};

use crate::core_trait::CoreZswapPool;
use crate::error::*;
use crate::utils::*;

// mod callback;
pub mod core_trait;
mod error;
pub mod ft_receiver;
mod internal;
mod manager;
pub mod utils;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    factory: AccountId,
    token_0: AccountId,
    token_1: AccountId,
    deposited_token_0: LookupMap<AccountId, u128>,
    deposited_token_1: LookupMap<AccountId, u128>,
    approved_token_0: LookupMap<AccountId, AccountId>, // improve to use HashMap later
    approved_token_1: LookupMap<AccountId, AccountId>, // improve to use HashMap later

    tick_spacing: u32,
    fee: u32,

    fee_growth_global_0_x128: u128,
    fee_growth_global_1_x128: u128,

    slot_0: Slot0,
    liquidity: u128,

    ticks: LookupMap<i32, TickInfo>,
    tick_bitmap: LookupMap<i16, U256>,
    positions: LookupMap<CryptoHash, PositionInfo>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Pools,
    Accounts,
    FeeToTickSpacing,
    Shares { pool_id: u32 },
    DepositedToken { token_id: AccountId },
    ApprovedToken { token_id: AccountId },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolView {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        token_0: AccountId,
        token_1: AccountId,
        tick_spacing: u32,
        fee: u32,
        sqrt_price_x96: U128,
    ) -> Self {
        Self {
            factory: env::predecessor_account_id(),
            token_0: token_0.clone(),
            token_1: token_1.clone(),
            deposited_token_0: LookupMap::new(StorageKey::DepositedToken {
                token_id: token_0.clone(),
            }),
            deposited_token_1: LookupMap::new(StorageKey::DepositedToken {
                token_id: token_1.clone(),
            }),
            approved_token_0: LookupMap::new(StorageKey::ApprovedToken { token_id: token_0 }),
            approved_token_1: LookupMap::new(StorageKey::ApprovedToken { token_id: token_1 }),

            tick_spacing,
            fee,
            fee_growth_global_0_x128: 0,
            fee_growth_global_1_x128: 0,
            slot_0: Slot0 {
                sqrt_price_x96,
                tick: tick_math::get_tick_at_sqrt_ratio(U256::from(sqrt_price_x96.0)),
            },
            liquidity: 0,
            ticks: LookupMap::new(StorageKey::Pools),
            tick_bitmap: LookupMap::new(StorageKey::Pools),
            positions: LookupMap::new(StorageKey::Pools),
        }
    }

    #[payable]
    pub fn initialize(&mut self, sqrt_price_x96: U128) {
        if self.slot_0.sqrt_price_x96.0 != 0 {
            env::panic_str(ALREADY_INITIALIZED);
        }
        let tick = tick_math::get_tick_at_sqrt_ratio(U256::from(sqrt_price_x96.0));

        self.slot_0 = Slot0 {
            sqrt_price_x96,
            tick,
        };
    }
}

// Implement the contract structure
#[near_bindgen]
impl CoreZswapPool for Contract {
    /// Mint liquidity for the given account
    ///
    /// Note: This function is not called by user directly, but by ZswapManager
    #[payable]
    fn mint(
        &mut self,
        payer: AccountId,
        recipient: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount: U128,
    ) -> [U128; 2] {
        if lower_tick >= upper_tick
            || lower_tick < TickConstants::MIN_TICK
            || upper_tick > TickConstants::MAX_TICK
        {
            env::panic_str(INVALID_TICK_RANGE);
        }

        if amount.0 == 0 {
            env::panic_str(ZERO_LIQUIDITY);
        }
        let amounts = self.modify_position(&recipient, lower_tick, upper_tick, amount.0 as i128);
        let amount_0 = amounts[0] as u128;
        let amount_1 = amounts[1] as u128;
        log!("Used amount_0: {}", amount_0);
        log!("Used amount_1: {}", amount_1);

        if amount_0 > 0 {
            self.internal_collect_token_0_to_mint(&payer, &env::predecessor_account_id(), amount_0);
        }

        if amount_1 > 0 {
            self.internal_collect_token_1_to_mint(&payer, &env::predecessor_account_id(), amount_1);
        }

        [U128::from(amount_0), U128::from(amount_1)]
    }

    #[payable]
    fn swap(
        &mut self,
        recipient: AccountId,
        zero_for_one: bool,
        amount_specified: U128,
        sqrt_price_limit_x96: Option<U128>,
    ) -> PromiseOrValue<U128> {
        let sqrt_price_limit_x96 = match sqrt_price_limit_x96 {
            Some(sqrt_price_limit) => {
                if zero_for_one && (sqrt_price_limit.0 > self.slot_0.sqrt_price_x96.0) {
                    env::panic_str(INVALID_PRICE_LIMIT);
                }

                if !zero_for_one && (sqrt_price_limit.0 < self.slot_0.sqrt_price_x96.0) {
                    env::panic_str(INVALID_PRICE_LIMIT);
                }

                U256::from(sqrt_price_limit.0).as_u160()
            }
            None => {
                if zero_for_one {
                    TickConstants::min_sqrt_ratio()
                } else {
                    TickConstants::max_sqrt_ratio()
                }
            }
        };

        let mut state = SwapState {
            amount_specified_remaining: amount_specified.0,
            amount_calculated: 0,
            sqrt_price_x96: self.slot_0.sqrt_price_x96.0,
            tick: self.slot_0.tick,
            fee_growth_global_x128: if zero_for_one {
                self.fee_growth_global_0_x128
            } else {
                self.fee_growth_global_1_x128
            },
            liquidity: self.liquidity,
        };

        while state.amount_specified_remaining > 0
            && state.sqrt_price_x96 != sqrt_price_limit_x96.as_u128()
        {
            let mut step = StepState::default();

            (step.next_tick, _) = tick_bitmap::next_initialized_tick_within_one_word(
                &self.tick_bitmap,
                state.tick,
                self.tick_spacing as i32,
                zero_for_one,
            );

            step.sqrt_price_start_x96 = state.sqrt_price_x96;
            step.sqrt_price_next_x96 = tick_math::get_sqrt_ratio_at_tick(step.next_tick).as_u128();

            let sqrt_target_price_x96 = if (zero_for_one
                && step.sqrt_price_next_x96 < sqrt_price_limit_x96.as_u128())
                || (!zero_for_one && step.sqrt_price_next_x96 > sqrt_price_limit_x96.as_u128())
            {
                sqrt_price_limit_x96
            } else {
                U256::from(step.sqrt_price_next_x96).as_u160()
            };

            let (sqrt_price_x96, amount_in, amount_out, fee_amount) = swap_math::compute_swap_step(
                U256::from(state.sqrt_price_x96),
                sqrt_target_price_x96,
                state.liquidity,
                I256::from(state.amount_specified_remaining),
                self.fee,
            );

            (
                state.sqrt_price_x96,
                step.amount_in,
                step.amount_out,
                step.fee_amount,
            ) = (
                sqrt_price_x96.as_u128(),
                amount_in.as_u128(),
                amount_out.as_u128(),
                fee_amount.as_u128(),
            );

            state.amount_specified_remaining -= step.amount_in + step.fee_amount;
            state.amount_calculated += step.amount_out;

            if state.liquidity > 0 {
                state.fee_growth_global_x128 += FullMath::mul_div(
                    U256::from(step.fee_amount),
                    fixed_point_128::get_q128(),
                    U256::from(state.liquidity),
                )
                .as_u128();
            }

            if state.sqrt_price_x96 == step.sqrt_price_next_x96 {
                let mut tick = self.ticks.get(&step.next_tick).unwrap_or_default();

                let fee_growth_global_0_x128 = if zero_for_one {
                    state.fee_growth_global_x128
                } else {
                    self.fee_growth_global_0_x128
                };

                let fee_growth_global_1_x128 = if zero_for_one {
                    self.fee_growth_global_1_x128
                } else {
                    state.fee_growth_global_x128
                };

                let mut liquidity_delta =
                    tick.cross(fee_growth_global_0_x128, fee_growth_global_1_x128);
                self.ticks.insert(&step.next_tick, &tick);

                if zero_for_one {
                    liquidity_delta = -liquidity_delta;
                }

                state.liquidity = liquidity_math::add_delta(state.liquidity, liquidity_delta);

                if state.liquidity == 0 {
                    env::panic_str(NOT_ENOUGH_LIQUIDITY)
                }

                state.tick = if zero_for_one {
                    step.next_tick - 1
                } else {
                    step.next_tick
                }
            } else if state.sqrt_price_x96 != step.sqrt_price_next_x96 {
                state.tick = tick_math::get_tick_at_sqrt_ratio(U256::from(state.sqrt_price_x96));
            }
        }

        if state.tick != self.slot_0.tick {
            self.slot_0.sqrt_price_x96 = U128::from(state.sqrt_price_x96);
            self.slot_0.tick = state.tick;
        } else {
            self.slot_0.sqrt_price_x96 = U128::from(state.sqrt_price_x96);
        }

        if self.liquidity != state.liquidity {
            self.liquidity = state.liquidity;
        }

        if zero_for_one {
            self.fee_growth_global_0_x128 = state.fee_growth_global_x128;
        } else {
            self.fee_growth_global_1_x128 = state.fee_growth_global_x128;
        }

        let amount_in = amount_specified.0 - state.amount_specified_remaining;
        let amount_out = state.amount_calculated;
        let caller = env::predecessor_account_id();

        if zero_for_one {
            let deposited_token_0 = self.deposited_token_0.get(&caller).unwrap_or_default();
            if deposited_token_0 < amount_in {
                env::panic_str(INSUFFICIENT_INPUT_AMOUNT);
            }
            self.deposited_token_0
                .insert(&caller, &(deposited_token_0 - amount_in));

            ext_ft_core::ext(self.token_1.clone())
                .with_attached_deposit(ONE_YOCTO)
                .ft_transfer(recipient, U128::from(amount_out), None);
        } else {
            let deposited_token_1 = self.deposited_token_1.get(&caller).unwrap_or_default();
            if deposited_token_1 < amount_in {
                env::panic_str(INSUFFICIENT_INPUT_AMOUNT);
            }
            self.deposited_token_1
                .insert(&caller, &(deposited_token_1 - amount_in));

            ext_ft_core::ext(self.token_0.clone())
                .with_attached_deposit(ONE_YOCTO)
                .ft_transfer(recipient, U128::from(amount_out), None);
        }

        PromiseOrValue::Value(U128::from(amount_out))
    }

    #[payable]
    fn burn(&mut self, lower_tick: i32, upper_tick: i32, amount: U128) -> [U128; 2] {
        if lower_tick >= upper_tick
            || lower_tick < TickConstants::MIN_TICK
            || upper_tick > TickConstants::MAX_TICK
        {
            env::panic_str(INVALID_TICK_RANGE);
        }

        if amount.0 == 0 {
            env::panic_str(ZERO_LIQUIDITY);
        }

        let owner = env::predecessor_account_id();
        let amounts = self.modify_position(&owner, lower_tick, upper_tick, -(amount.0 as i128));
        let amount_0 = amounts[0].unsigned_abs();
        let amount_1 = amounts[1].unsigned_abs();

        if amount_0 > 0 || amount_1 > 0 {
            let position_key = self.get_position_key(&owner, lower_tick, upper_tick);
            let mut position = self.positions.get(&position_key).unwrap();
            position.tokens_owed_0 += amount_0;
            position.tokens_owed_1 += amount_1;

            self.positions.insert(&position_key, &position);
        }

        [U128::from(amount_0), U128::from(amount_1)]
    }

    #[payable]
    fn collect(
        &mut self,
        recipient: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount_0_requested: U128,
        amount_1_requested: U128,
    ) -> [U128; 2] {
        if lower_tick >= upper_tick
            || lower_tick < TickConstants::MIN_TICK
            || upper_tick > TickConstants::MAX_TICK
        {
            env::panic_str(INVALID_TICK_RANGE);
        }

        let owner = env::predecessor_account_id();
        let position_key = self.get_position_key(&owner, lower_tick, upper_tick);
        let mut position = self.positions.get(&position_key).unwrap();

        let amount_0 = position.tokens_owed_0.min(amount_0_requested.0);
        let amount_1 = position.tokens_owed_1.min(amount_1_requested.0);

        log!("Collected amount 0: {}", amount_0);
        log!("Collected amount 1: {}", amount_1);

        if amount_0 > 0 {
            position.tokens_owed_0 -= amount_0;
            ext_ft_core::ext(self.token_0.clone())
                .with_attached_deposit(ONE_YOCTO)
                .ft_transfer(recipient.clone(), amount_0.into(), None);
        }

        if amount_1 > 0 {
            position.tokens_owed_1 -= amount_1;
            ext_ft_core::ext(self.token_1.clone())
                .with_attached_deposit(ONE_YOCTO)
                .ft_transfer(recipient, amount_1.into(), None);
        }

        self.positions.insert(&position_key, &position);

        [U128::from(amount_0), U128::from(amount_1)]
    }

    fn get_slot_0(&self) -> Slot0 {
        self.slot_0.clone()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {}
