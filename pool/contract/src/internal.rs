use near_sdk::{ env, serde_json::json, Gas, Promise, AccountId };

use crate::Contract;

impl Contract {
    pub fn get_balance0_promise(&self) -> Promise {
        let args = json!({ "account_id": env::current_account_id() }).to_string().into_bytes();

        Promise::new(self.token0.clone()).function_call("ft_balance_of".to_owned(), args, 0, Gas::default())
    }

    pub fn get_balance1_promise(&self) -> Promise {
        let args = json!({ "account_id": env::current_account_id() }).to_string().into_bytes();

        Promise::new(self.token1.clone()).function_call("ft_balance_of".to_owned(), args, 0, Gas::default())
    }

    pub fn modify_position(
        &self,
        address: AccountId,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: i128
    ) -> (String, i128, i128) {
        let return_message = format!(
            "modify_position({}, {}, {}, {})",
            address,
            lower_tick,
            upper_tick,
            liquidity_delta
        );
        (return_message, 0, 0)
    }
}
