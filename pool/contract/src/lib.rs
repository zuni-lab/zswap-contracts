use ethnum::U256;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, CryptoHash, Promise};

use crate::account::Account;
use crate::manager::ext_ft_zswap_manager;
use crate::utils::*;

mod account;
mod callback;
mod error;
mod internal;
mod manager;
mod utils;

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
        lower_tick: u32,
        upper_tick: u32,
        amount: u128,
        data: Vec<u8>,
    ) -> Promise {
        // === IMPLEMENT HERE ===
        todo!("calculate amount0 and amount1");
        let amount0 = 0;
        let amount1 = 0;
        // ======================

        let zswap_manager = env::predecessor_account_id();

        let amount0_before_promise = self.get_balance0_promise();
        let amount1_before_promise = self.get_balance1_promise();

        amount0_before_promise
            .and(amount1_before_promise)
            .and(
                ext_ft_zswap_manager::ext(zswap_manager)
                    .transfer_approved_tokens_to_mint(amount0, amount1, data),
            )
            .then(Self::ext(env::current_account_id()).mint_callback_post_tokens_transfer())
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
