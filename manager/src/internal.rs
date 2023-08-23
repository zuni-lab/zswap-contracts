use std::cmp::Ordering;

use near_sdk::{env, AccountId, Promise};
use zswap_math_library::pool_account;
// use zswap_pool::core_trait::ext_zswap_pool_core;

use crate::error::TOKENS_MUST_BE_DIFFERENT;
// use crate::ft_account::Account;
use crate::pool::ext_zswap_pool;
use crate::utils::SwapCallbackData;
use crate::Contract;

impl Contract {
    // pub fn internal_deposit(
    //     &mut self,
    //     sender_id: &AccountId,
    //     token_id: &AccountId,
    //     amount: Balance,
    // ) {
    //     let mut account = self.get_account(sender_id);

    //     if amount > 0 {
    //         let current_amount = account.deposited_tokens.get(token_id).unwrap_or(0);
    //         account
    //             .deposited_tokens
    //             .insert(token_id, &(current_amount + amount));
    //     }

    //     // save account
    //     self.accounts.insert(sender_id, &account);
    // }

    pub fn internal_swap(
        &mut self,
        amount_in: u128,
        recipient: AccountId,
        sqrt_price_limit_x96: u128,
        data: SwapCallbackData,
    ) -> Promise {
        let zero_for_one = data.token_0 < data.token_1;
        let pool = self.get_pool(&data.token_0, &data.token_1, data.fee);
        let encoded_data = near_sdk::serde_json::to_vec(&data).unwrap();

        ext_zswap_pool::ext(pool)
            .swap(
                recipient,
                zero_for_one,
                amount_in.into(),
                sqrt_price_limit_x96.into(),
                encoded_data,
            )
            .then(Self::ext(env::current_account_id()).calculate_amount_out(zero_for_one))
    }

    // ========= VIEW METHODS =========

    pub fn get_pool(&self, token_0: &AccountId, token_1: &AccountId, fee: u32) -> AccountId {
        let ordered_token_0;
        let ordered_token_1;
        match token_0.cmp(token_1) {
            Ordering::Less => {
                ordered_token_0 = token_0;
                ordered_token_1 = token_1;
            }
            Ordering::Greater => {
                ordered_token_0 = token_1;
                ordered_token_1 = token_0;
            }
            Ordering::Equal => env::panic_str(TOKENS_MUST_BE_DIFFERENT),
        }

        pool_account::compute_account(
            self.factory.clone(),
            ordered_token_0.clone(),
            ordered_token_1.clone(),
            fee,
        )
    }
}
