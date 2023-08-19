use near_sdk::{ext_contract, json_types::U128, AccountId, Promise};

use crate::utils::Slot0;

#[ext_contract(ext_zswap_pool_core)]
pub trait CoreZswapPool {
    fn mint(
        &mut self,
        owner: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount: U128,
        data: Vec<u8>,
    ) -> Promise;

    fn swap(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
        amount_out_min: u128,
        recipient: AccountId,
    );

    fn burn(&mut self, from: AccountId, amount: u128);

    fn collect(&mut self, token_in: AccountId);

    fn get_slot_0(&self) -> Slot0;
}
