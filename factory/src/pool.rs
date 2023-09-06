use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ext_contract, AccountId};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolView {
    pub pool_id: AccountId,
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub tick_spacing: u32,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInitArgs {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub tick_spacing: u32,
    pub fee: u32,
}

#[ext_contract(ext_zswap_pool)]
pub trait FtZswapPool {
    fn new(
        token_0: AccountId,
        token_1: AccountId,
        tick_spacing: u32,
        fee: u32,
        sqrt_price_x96: U128,
    ) -> Self;
}
