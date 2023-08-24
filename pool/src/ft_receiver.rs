use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{env, json_types::U128, log, near_bindgen, AccountId, PromiseOrValue};

use crate::{error::UNSUPPORTED_TOKEN, Contract, ContractExt};

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = &env::predecessor_account_id();
        if msg.is_empty() {
            if token_in == &self.token_0 {
                self.token_0_deposit(&sender_id, amount.into());
            } else if token_in == &self.token_1 {
                self.token_1_deposit(&sender_id, amount.into());
            } else {
                env::panic_str(UNSUPPORTED_TOKEN)
            }
            let unused_amount = U128(0);
            PromiseOrValue::Value(unused_amount)
        } else {
            // TODO: handle swap
            log!("handle deposit swap here");
            PromiseOrValue::Value(U128(0))
        }
    }
}

impl Contract {
    fn token_0_deposit(&mut self, sender_id: &AccountId, amount: u128) {
        let depositted_token_opt = self.depositted_token_0.get(sender_id);
        match depositted_token_opt {
            Some(deposistted) => {
                self.depositted_token_0
                    .insert(sender_id, &(deposistted + amount));
            }
            None => {
                self.depositted_token_0.insert(sender_id, &amount);
            }
        }
    }

    fn token_1_deposit(&mut self, sender_id: &AccountId, amount: u128) {
        let depositted_token_opt = self.depositted_token_1.get(sender_id);
        match depositted_token_opt {
            Some(deposistted) => {
                self.depositted_token_1
                    .insert(sender_id, &(deposistted + amount));
            }
            None => {
                self.depositted_token_1.insert(sender_id, &amount);
            }
        }
    }
}
