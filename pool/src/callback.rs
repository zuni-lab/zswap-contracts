use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, Promise, PromiseError};

use crate::error::{CAN_NOT_COLLECT_TOKENS, INSUFFICIENT_INPUT_AMOUNT};
use crate::Contract;
use crate::ContractExt;

pub trait PoolCallback {
    fn mint_callback_post_collected_tokens(
        &self,
        balance_0_before_res: Result<U128, PromiseError>,
        balance_1_before_res: Result<U128, PromiseError>,
        collected_tokens_res: Result<(), PromiseError>,
        amount_0: u128,
        amount_1: u128,
    ) -> Promise;

    fn check_mint_callback(
        &self,
        balance_0_after_res: Result<U128, PromiseError>,
        balance_1_after_res: Result<U128, PromiseError>,
        balance_0_before: u128,
        balance_1_before: u128,
        amount_0: u128,
        amount_1: u128,
    ) -> [U128; 2];

    fn swap_callback_post_collected_token(amount: Result<U128, PromiseError>, zero_for_one: bool);
}

#[near_bindgen]
impl PoolCallback for Contract {
    #[private]
    fn mint_callback_post_collected_tokens(
        &self,
        #[callback_result] amount_0_before_res: Result<U128, PromiseError>,
        #[callback_result] amount_1_before_res: Result<U128, PromiseError>,
        #[callback_result] collected_tokens_res: Result<(), PromiseError>,
        amount_0: u128,
        amount_1: u128,
    ) -> Promise {
        if collected_tokens_res.is_err() {
            env::panic_str(&format!(
                "{}: {:?}",
                CAN_NOT_COLLECT_TOKENS,
                collected_tokens_res.unwrap_err()
            ))
        }

        self.get_balance_0_promise()
            .and(self.get_balance_1_promise())
            .then(Self::ext(env::current_account_id()).check_mint_callback(
                amount_0_before_res.unwrap().0,
                amount_1_before_res.unwrap().0,
                amount_0,
                amount_1,
            ))
    }

    #[private]
    fn check_mint_callback(
        &self,
        #[callback_result] balance_0_after_res: Result<U128, PromiseError>,
        #[callback_result] balance_1_after_res: Result<U128, PromiseError>,
        balance_0_before: u128,
        balance_1_before: u128,
        amount_0: u128,
        amount_1: u128,
    ) -> [U128; 2] {
        if amount_0 > 0 && balance_0_before + amount_0 > balance_0_after_res.unwrap().0 {
            env::panic_str(INSUFFICIENT_INPUT_AMOUNT)
        }

        if amount_1 > 0 && balance_1_before + amount_1 > balance_1_after_res.unwrap().0 {
            env::panic_str(INSUFFICIENT_INPUT_AMOUNT)
        }

        [U128::from(amount_0), U128::from(amount_1)]
    }

    #[allow(unused)]
    #[private]
    fn swap_callback_post_collected_token(
        #[callback_result] amount: Result<U128, PromiseError>,
        zero_for_one: bool,
    ) {
        todo!("check input amount is enough or not")
    }
}
