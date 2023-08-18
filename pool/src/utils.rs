use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

// First slot will contain essential data
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Slot0 {
    // Current sqrt(P)
    pub sqrt_price_x96: u128,
    // Current tick
    pub tick: i32,
}

pub struct SwapState {
    amount_specified_remaining: u128,
    amount_calculated: u128,
    sqrt_price_x96: u128,
    tick: i32,
    fee_growth_global_x128: u128,
    liquidity: u128,
}

pub struct StepState {
    sqrt_price_start_x96: u128,
    next_tick: i32,
    initialized: bool,
    sqrt_price_next_x96: u128,
    amount_in: u128,
    amount_out: u128,
    fee_amount: u128,
}
