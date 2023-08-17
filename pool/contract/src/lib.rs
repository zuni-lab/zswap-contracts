use ethnum::U256;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, CryptoHash, Promise};

use crate::account::Account;
use crate::error::{INVALID_TICK_RANGE, ZERO_LIQUIDITY};
use crate::manager::ext_ft_zswap_manager;
use crate::utils::*;

use zswap_math_library::TickMath;

mod account;
mod callback;
mod error;
mod internal;
mod manager;
mod utils;

// TODO: remove this
struct Tick;
struct Position;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    factory: AccountId,
    token0: AccountId,
    token1: AccountId,
    tick_spacing: u32,
    fee: u32,

    fee_growth_global0_x128: u128,
    fee_growth_global1_x128: u128,

    slot0: Slot0,
    liquidity: u128,

    //TODO: import Tick and Position from lib
    ticks: LookupMap<i32, Tick>, // import from `lib`
    tick_bitmap: LookupMap<i16, U256>,
    positions: LookupMap<CryptoHash, Position>, // import from `lib`

    /// Accounts registered, keeping track all the amounts deposited, storage and more.
    accounts: LookupMap<AccountId, Account>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Pools,
    Accounts,
    FeeToTickSpacing,
    Shares { pool_id: u32 },
    AccountTokens { account_id: AccountId },
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        factory: AccountId,
        token0: AccountId,
        token1: AccountId,
        tick_spacing: u32,
        fee: u32,
    ) -> Self {
        Self {
            factory,
            token0,
            token1,
            tick_spacing,
            fee,
            fee_growth_global0_x128: 0,
            fee_growth_global1_x128: 0,
            slot0: Slot0 {
                sqrt_price_x96: 0,
                tick: 0,
            },
            liquidity: 0,
            ticks: LookupMap::new(StorageKey::Pools),
            tick_bitmap: LookupMap::new(StorageKey::Pools),
            positions: LookupMap::new(StorageKey::Pools),
            accounts: LookupMap::new(StorageKey::Accounts),
        }
    }

    #[payable]
    pub fn initialize(&mut self, sqrt_price_x96: U128) {
        todo!("initialize slot0");
    }

    /// Mint liquidity for the given account
    /// - `to` - the liquidity recipient
    /// - `amount` - the amount of liquidity
    /// Following those steps:
    /// 1. Calculate amount
    /// 2. Calling to ZswapManager to collect tokens
    /// 3. Callback to check collected amounts
    /// Note: This function is not called by user directly, but by ZswapManager
    #[payable]
    pub fn mint(
        &mut self,
        owner: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount: U128,
        data: Vec<u8>,
    ) -> Promise {
        let check1 = lower_tick >= upper_tick;
        let check2 = lower_tick < TickMath::TickConstants::MIN_TICK;
        let check3 = upper_tick > TickMath::TickConstants::MAX_TICK;
        if check1 || check2 || check3 {
            env::panic_str(INVALID_TICK_RANGE);
        }

        if amount.0 == 0 {
            env::panic_str(ZERO_LIQUIDITY);
        }
        let (_, amount_0_int, amount_1_int) =
            self.modify_position(owner, lower_tick, upper_tick, amount.0 as i128);

        let amount_0 = amount_0_int as u128;
        let amount_1 = amount_1_int as u128;

        let zswap_manager = env::predecessor_account_id();

        // let amount0_before_promise = if amount_0 > 0 {
        //     self.get_balance0_promise()
        // } else {
        //     Promise::new(zswap_manager.clone())
        // };

        // let amount1_before_promise = if amount_1 > 0 {
        //     self.get_balance1_promise()
        // } else {
        //     Promise::new(zswap_manager.clone())
        // };

        Promise::new(env::current_account_id())
            .and(self.get_balance0_promise()) // get balance of token0 before transfer
            .and(self.get_balance1_promise()) // get balance of token1 before transfer
            .and(
                ext_ft_zswap_manager::ext(zswap_manager)
                    .transfer_approved_tokens_to_mint(amount_0, amount_1, data),
            )
            .and(self.get_balance0_promise()) // get balance of token0 after transfer
            .and(self.get_balance1_promise()) // get balance of token1 after transfer
            .then(
                Self::ext(env::current_account_id())
                    .mint_callback_post_tokens_transfer(amount_0, amount_1),
            )
    }

    #[payable]
    pub fn burn(from: AccountId, amount: u128) {
        todo!("burn");
    }

    #[payable]
    pub fn swap(
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
        amount_out_min: u128,
        recipient: AccountId,
    ) {
        todo!("swap");
    }

    #[payable]
    pub fn collect(token_in: AccountId) {
        todo!("collect");
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn get_default_greeting() {
    //     let contract = Contract::default();
    //     // this test did not call set_greeting so should return the default "Hello" greeting
    //     assert_eq!(contract.get_greeting(), "Hello".to_string());
    // }

    // #[test]
    // fn set_then_get_greeting() {
    //     let mut contract = Contract::default();
    //     contract.set_greeting("howdy".to_string());
    //     assert_eq!(contract.get_greeting(), "howdy".to_string());
    // }
}
