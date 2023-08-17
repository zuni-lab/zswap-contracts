use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, PromiseError};

use crate::Contract;
use crate::ContractExt;

pub trait PoolCallback {
    fn mint_callback_post_tokens_transfer(
        amount0_before: Result<U128, PromiseError>,
        amount1_before: Result<U128, PromiseError>,
    );

    fn swap_callback_post_tokens_transfer(zero_for_one: bool, amount: Result<U128, PromiseError>);
}

#[near_bindgen]
impl PoolCallback for Contract {
    #[private]
    fn mint_callback_post_tokens_transfer(
        #[callback_result] amount0_before: Result<U128, PromiseError>,
        #[callback_result] amount1_before: Result<U128, PromiseError>,
    ) {
        todo!("check amount0 and amount1 is enough or not")
    }

    #[private]
    fn swap_callback_post_tokens_transfer(
        zero_for_one: bool,
        #[callback_result] amount: Result<U128, PromiseError>,
    ) {
        todo!("check input amount is enough or not")
    }
}
