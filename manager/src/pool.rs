use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Promise;
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
        amount: U128,
        data: Vec<u8>,
    ) -> Promise;

    fn swap(
        &mut self,
        recipient: AccountId,
        zero_for_one: bool,
        amount_specified: U128,
        sqrt_price_limit_x96: U128,
        data: Vec<u8>,
    );

    fn get_slot_0(&self) -> Slot0;
}
