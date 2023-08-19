use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ext_contract, json_types::U128, AccountId};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Slot0 {
    pub sqrt_price_x96: U128,
    pub tick: i32,
}

#[ext_contract(ext_zswap_pool)]
pub trait ZswapPool {
    fn mint(
        &mut self,
        owner: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        amount: u128,
        data: Vec<u8>,
    ) -> [U128; 2];

    fn swap(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
        amount_out_min: u128,
        recipient: AccountId,
    );

    fn get_slot_0(&self) -> Slot0;
}
