use std::cmp::Ordering;

use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance};

use crate::error::TOKENS_MUST_BE_DIFFERENT;
use crate::ft_account::Account;
use crate::Contract;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolCallbackData {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub payer: AccountId,
}

impl Contract {
    pub fn internal_deposit(
        &mut self,
        sender_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) {
        let mut account = self.get_account(sender_id);

        if amount > 0 {
            let current_amount = account.deposited_tokens.get(token_id).unwrap_or(0);
            account
                .deposited_tokens
                .insert(token_id, &(current_amount + amount));
        }

        // save account
        self.accounts.insert(sender_id, &account);
    }

    pub fn internal_swap(
        &mut self,
        amount_in: u128,
        recipient: AccountId,
        sqrt_price_limit_x96: u128,
        data: PoolCallbackData,
    ) -> u128 {
        let amount_out = 0;
        amount_out
    }

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

        let hash_data = env::keccak256(
            [
                ordered_token_0.as_bytes(),
                ordered_token_1.as_bytes(),
                &fee.to_le_bytes(),
            ]
            .concat()
            .as_slice(),
        );

        let subaccount: AccountId =
            format!("{:x?}.{}", &hash_data[0..8], env::current_account_id())
                .parse()
                .unwrap();

        subaccount
    }
}
