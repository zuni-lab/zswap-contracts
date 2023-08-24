use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

// First slot will contain essential data
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Slot0 {
    // Current sqrt(P)
    pub sqrt_price_x96: U128,
    // Current tick
    pub tick: i32,
}

pub struct SwapState {
    pub amount_specified_remaining: u128,
    pub amount_calculated: u128,
    pub sqrt_price_x96: u128,
    pub tick: i32,
    pub fee_growth_global_x128: u128,
    pub liquidity: u128,
}

#[derive(Default)]
pub struct StepState {
    pub sqrt_price_start_x96: u128,
    pub next_tick: i32,
    pub initialized: bool,
    pub sqrt_price_next_x96: u128,
    pub amount_in: u128,
    pub amount_out: u128,
    pub fee_amount: u128,
}
