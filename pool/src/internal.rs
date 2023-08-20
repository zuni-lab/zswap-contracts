use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{env, json_types::I128, AccountId, Promise};

use crate::Contract;

impl Contract {
    pub fn get_balance0_promise(&self) -> Promise {
        ext_ft_core::ext(self.token_0.clone()).ft_balance_of(env::current_account_id())
    }

    pub fn get_balance1_promise(&self) -> Promise {
        ext_ft_core::ext(self.token_1.clone()).ft_balance_of(env::current_account_id())
    }

    pub fn modify_position(
        &mut self,
        address: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: i128,
    ) -> (String, i128, i128) {
        // let position_key = env::keccak256(&[1, 2, 3]);
        // let position = self.positions.get(&position_key.into()).unwrap_or_default();
        // let current_tick = self.slot_0.tick;
        // let tick_info = self.ticks.get(&current_tick).unwrap_or_default();
        // tick_info.update();

        let return_message = format!(
            "modify_position({}, {}, {}, {})",
            address, lower_tick, upper_tick, liquidity_delta
        );
        (return_message, 0, 0)
    }
}
