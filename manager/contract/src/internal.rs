use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

use crate::Contract;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolCallbackData {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub payer: AccountId,
}

impl Contract {
    pub fn internal_swap(
        &self,
        amount_in: u128,
        recipient: AccountId,
        sqrt_price_limit_x96: u128,
        data: PoolCallbackData,
    ) -> u128 {
        let amount_out = 0;
        amount_out
    }

    pub fn get_pool(&self, token_0: &AccountId, token_1: &AccountId, fee: u32) -> AccountId {
        AccountId::new_unchecked("account_id".to_string())
    }
}
