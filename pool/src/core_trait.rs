use near_sdk::{ext_contract, json_types::U128, AccountId, PromiseOrValue};

use crate::utils::Slot0;

#[ext_contract(ext_zswap_pool_core)]
pub trait CoreZswapPool {
    fn mint(
        &mut self,
        owner: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount: U128,
        // data: Vec<u8>,
    ) -> [U128; 2];

    fn swap(
        &mut self,
        recipient: AccountId,
        zero_for_one: bool,
        amount_specified: U128,
        sqrt_price_limit_x96: Option<U128>,
    ) -> PromiseOrValue<U128>;

    fn burn(&mut self, lower_tick: i32, upper_tick: i32, amount: U128) -> [U128; 2];

    fn collect(
        &mut self,
        recipient: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount_0_requested: U128,
        amount_1_requested: U128,
    ) -> [U128; 2];

    fn get_slot_0(&self) -> Slot0;
}
