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

pub struct SwapSingleParams {
    token_in: AccountId,
    token_out: AccountId,
    fee: u32,
    amount_in: u128,
    sqrt_price_limit_x96: u128,
}

pub struct SwapParams {
    tokens: Vec<AccountId>,
    fees: Vec<u32>,
    recipient: AccountId,
    amount_in: u128,
    amount_out_min: u128,
}

pub struct SwapCallbackData {
    token_0: AccountId,
    token_1: AccountId,
    payer: AccountId,
}
