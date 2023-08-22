// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, CryptoHash, PanicOnDefault};

use zswap_math_library::num160::AsU160;
use zswap_math_library::num256::U256;
use zswap_math_library::position::PositionInfo;
use zswap_math_library::tick::TickInfo;
use zswap_math_library::tick_math;
use zswap_math_library::tick_math::TickConstants;

use crate::core_trait::CoreZswapPool;
use crate::error::*;
use crate::utils::*;

// mod callback;
pub mod core_trait;
mod error;
mod ft_receiver;
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
    depositted_token_0: LookupMap<AccountId, u128>,
    depositted_token_1: LookupMap<AccountId, u128>,
    tick_spacing: u32,
    fee: u32,

    fee_growth_global_0_x128: U256,
    fee_growth_global_1_x128: U256,

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
    DeposittedToken { token_id: AccountId },
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
    pub fn new(token_0: AccountId, token_1: AccountId, tick_spacing: u32, fee: u32) -> Self {
        Self {
            factory: env::predecessor_account_id(),
            token_0: token_0.clone(),
            token_1: token_1.clone(),
            depositted_token_0: LookupMap::new(StorageKey::DeposittedToken { token_id: token_0 }),
            depositted_token_1: LookupMap::new(StorageKey::DeposittedToken { token_id: token_1 }),

            tick_spacing,
            fee,
            fee_growth_global_0_x128: U256::from(0),
            fee_growth_global_1_x128: U256::from(0),
            slot_0: Slot0 {
                sqrt_price_x96: U128::from(0),
                tick: 0,
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
        let tick = tick_math::get_tick_at_sqrt_ratio(U256::from(sqrt_price_x96.0).as_u160());

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
    /// - `to` - the liquidity recipient
    /// - `amount` - the amount of liquidity
    /// Following those steps:
    /// 1. Calculate amount
    /// 2. Calling to ZswapManager callback to check slippage
    /// Note: This function is not called by user directly, but by ZswapManager
    #[payable]
    fn mint(
        &mut self,
        owner: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount: U128,
        // data: Vec<u8>,
    ) -> [U128; 2] {
        let check1 = lower_tick >= upper_tick;
        let check2 = lower_tick < TickConstants::MIN_TICK;
        let check3 = upper_tick > TickConstants::MAX_TICK;
        if check1 || check2 || check3 {
            env::panic_str(INVALID_TICK_RANGE);
        }

        if amount.0 == 0 {
            env::panic_str(ZERO_LIQUIDITY);
        }
        let amounts = self.modify_position(owner.clone(), lower_tick, upper_tick, amount.0 as i128);
        let amount_0 = amounts[0] as u128;
        let amount_1 = amounts[1] as u128;

        if amount_0 > 0 {
            self.internal_collect_token_0_to_mint(&owner, amount_0);
        }

        if amount_1 > 0 {
            self.internal_collect_token_1_to_mint(&owner, amount_1);
        }

        [U128::from(amount_0), U128::from(amount_1)]
    }

    #[allow(unused)]
    #[payable]
    fn swap(
        &mut self,
        recipient: AccountId,
        zero_for_one: bool,
        amount_specified: U128,
        sqrt_price_limit_x96: U128,
        data: Vec<u8>,
    ) {
        todo!("swap");
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
