use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, PromiseError};

use crate::error::INSUFFICIENT_INPUT_AMOUNT;
use crate::Contract;
use crate::ContractExt;

pub trait PoolCallback {
    fn mint_callback_post_collected_tokens(
        amount_0_before_res: Result<U128, PromiseError>,
        amount_1_before_res: Result<U128, PromiseError>,
        collected_tokens_res: Result<(), PromiseError>,
        amount_0_after_res: Result<U128, PromiseError>,
        amount_1_after_res: Result<U128, PromiseError>,
        amount_0: u128,
        amount_1: u128,
    ) -> [U128; 2];

    fn swap_callback_post_collected_token(amount: Result<U128, PromiseError>, zero_for_one: bool);
}

#[near_bindgen]
impl PoolCallback for Contract {
    #[private]
    fn mint_callback_post_collected_tokens(
        #[callback_result] amount_0_before_res: Result<U128, PromiseError>,
        #[callback_result] amount_1_before_res: Result<U128, PromiseError>,
        #[callback_result] _collected_tokens_res: Result<(), PromiseError>,
        #[callback_result] amount_0_after_res: Result<U128, PromiseError>,
        #[callback_result] amount_1_after_res: Result<U128, PromiseError>,
        amount_0: u128,
        amount_1: u128,
    ) -> [U128; 2] {
        let amount_0_before = amount_0_before_res.unwrap().0;
        let amount_1_before = amount_1_before_res.unwrap().0;
        let amount_0_after = amount_0_after_res.unwrap().0;
        let amount_1_after = amount_1_after_res.unwrap().0;

        if amount_0 > 0 && amount_0_before + amount_0 > amount_0_after {
            env::panic_str(INSUFFICIENT_INPUT_AMOUNT)
        }

        if amount_1 > 0 && amount_1_before + amount_1 > amount_1_after {
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
