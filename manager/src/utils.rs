use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

pub struct GetPositionParams {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub owner: AccountId,
    pub lower_tick: i32,
    pub upper_tick: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MintParams {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub amount_0_desired: U128,
    pub amount_1_desired: U128,
    pub amount_0_min: U128,
    pub amount_1_min: U128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapSingleParams {
    pub token_in: AccountId,
    pub token_out: AccountId,
    pub fee: u32,
    pub amount_in: u128,
    pub sqrt_price_limit_x96: u128,
}

pub struct SwapParams {
    tokens: Vec<AccountId>,
    fees: Vec<u32>,
    recipient: AccountId,
    amount_in: u128,
    amount_out_min: u128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapCallbackData {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub payer: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolCallbackData {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub payer: AccountId,
}
