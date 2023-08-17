use internal::PoolCallbackData;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, PromiseError};
use pool::Slot0;
use utils::MintParams;

use crate::pool::ext_zswap_pool;

mod callback;
mod error;
mod internal;
mod pool;
mod token_receiver;
mod utils;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    factory: AccountId,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(factory: AccountId) -> Self {
        Self { factory }
    }

    pub fn get_position(
        &self,
        token_1: AccountId,
        token_2: AccountId,
        fee: u32,
        owner: AccountId,
        lower_tick: i32,
        upperTick: i32,
    ) {
    }

    #[payable]
    pub fn mint(&mut self, params: MintParams) -> Promise {
        let pool = self.get_pool(&params.token_0, &params.token_1, params.fee);

        ext_zswap_pool::ext(pool).get_slot_0().then(
            Self::ext(env::current_account_id())
                .calculate_liquidity(env::predecessor_account_id(), params),
        )
    }

    #[payable]
    pub fn swap_single(
        token_in: AccountId,
        token_out: AccountId,
        fee: u32,
        lower_tick: i32,
        upper_tick: i32,
        amount_0_desired: u128,
        amount_1_desired: u128,
        amount_0_min: u128,
        amount_1_min: u128,
    ) {
    }

    #[payable]
    pub fn swap(
        tokens: Vec<AccountId>,
        fees: Vec<u32>,
        recipient: AccountId,
        amount_in: u128,
        amount_out_min: u128,
    ) {
    }

    #[private]
    pub fn calculate_liquidity(
        &mut self,
        #[callback_result] slot0: Result<Slot0, PromiseError>,
        recipient: AccountId,
        params: MintParams,
    ) -> Promise {
        let sqrt_price_x96 = slot0.unwrap().sqrt_price_x96;
        let sqrt_price_lower_x96 = 0u128; // TODO: Add TickMath.getSqrtRatioAtTick
        let sqrt_price_upper_x96 = 0u128; // TODO: Add TickMath.getSqrtRatioAtTick
        let liquidity = 0u128; // TODO: Add TickMath.getLiquidityForAmounts

        let pool_callback_data = PoolCallbackData {
            token_0: params.token_0,
            token_1: params.token_1,
            payer: recipient.clone(),
        };
        let data = near_sdk::serde_json::to_vec(&pool_callback_data).unwrap();

        ext_zswap_pool::ext(env::current_account_id())
            .mint(
                recipient,
                params.lower_tick,
                params.upper_tick,
                liquidity,
                data,
            )
            .then(
                Self::ext(env::current_account_id())
                    .manager_mint_callback(params.amount_0_min, params.amount_1_min),
            )
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
