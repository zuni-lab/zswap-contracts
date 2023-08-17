use near_sdk::{env, serde_json::json, Gas, Promise};

use crate::Contract;

impl Contract {
    pub fn get_balance0_promise(&self) -> Promise {
        let args = json!({ "account_id": env::current_account_id() })
            .to_string()
            .into_bytes();

        Promise::new(self.token0).function_call("ft_balance_of".to_owned(), args, 0, Gas::default())
    }

    pub fn get_balance1_promise(&self) -> Promise {
        let args = json!({ "account_id": env::current_account_id() })
            .to_string()
            .into_bytes();

        Promise::new(self.token1).function_call("ft_balance_of".to_owned(), args, 0, Gas::default())
    }
}
