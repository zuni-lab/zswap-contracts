use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, PromiseError};

use crate::error::{MINT_CALLBACK_ERROR, SLIPPAGE_CHECK_FAILED};
use crate::Contract;
use crate::ContractExt;

pub trait ManagerCallback {
    fn mint_callback(
        used_amounts_res: Result<[U128; 2], PromiseError>,
        amount_0_min: u128,
        amount_1_min: u128,
    ) -> [U128; 2];

    fn manager_swap_callback(amount_out_min: U128, amount_received: Result<U128, PromiseError>);
}

#[near_bindgen]
impl ManagerCallback for Contract {
    #[private]
    fn mint_callback(
        #[callback_result] used_amounts_res: Result<[U128; 2], PromiseError>,
        amount_0_min: u128,
        amount_1_min: u128,
    ) -> [U128; 2] {
        if used_amounts_res.is_err() {
            env::panic_str(MINT_CALLBACK_ERROR)
        }

        let received_amounts = used_amounts_res.unwrap();
        let amount_0 = received_amounts[0];
        let amount_1 = received_amounts[1];

        if amount_0.0 < amount_0_min || amount_1.0 < amount_1_min {
            env::panic_str(SLIPPAGE_CHECK_FAILED)
        }

        [amount_0, amount_1]
    }

    #[allow(unused)]
    #[private]
    fn manager_swap_callback(
        amount_out_min: U128,
        #[callback_result] amount_received: Result<U128, PromiseError>,
    ) {
        todo!("check output amount is enough or not")
    }
}
