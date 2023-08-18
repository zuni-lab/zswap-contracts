use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{env, json_types::U128, near_bindgen, AccountId, PromiseOrValue};

use crate::{Contract, ContractExt};

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        if msg.is_empty() {
            self.internal_deposit(&sender_id, &token_in, amount.into());
            PromiseOrValue::Value(amount)
        } else {
            // TODO: handle swap
            PromiseOrValue::Value(U128(0))
        }
    }
}
