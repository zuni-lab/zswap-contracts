use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, json_types::U128, near_bindgen, serde_json, AccountId, PromiseOrValue};

use crate::error::*;
use crate::{Contract, ContractExt};

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    Approve { account_id: AccountId },
}

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
            let message =
                serde_json::from_str::<TokenReceiverMessage>(&msg).expect(WRONG_MSG_FORMAT);
            match message {
                TokenReceiverMessage::Approve { account_id } => {
                    if token_in == &self.token_0 {
                        self.token_0_deposit(&sender_id, amount.into());
                        self.token_0_approve(&sender_id, &account_id);
                    } else if token_in == &self.token_1 {
                        self.token_1_deposit(&sender_id, amount.into());
                        self.token_1_approve(&sender_id, &account_id);
                    } else {
                        env::panic_str(UNSUPPORTED_TOKEN)
                    }

                    let unused_amount = U128(0);
                    PromiseOrValue::Value(unused_amount)
                }
            }
        }
    }
}

impl Contract {
    fn token_0_deposit(&mut self, sender_id: &AccountId, amount: u128) {
        let deposited_token_opt = self.deposited_token_0.get(sender_id);
        match deposited_token_opt {
            Some(deposited) => {
                self.deposited_token_0
                    .insert(sender_id, &(deposited + amount));
            }
            None => {
                self.deposited_token_0.insert(sender_id, &amount);
            }
        }
    }

    fn token_1_deposit(&mut self, sender_id: &AccountId, amount: u128) {
        let deposited_token_opt = self.deposited_token_1.get(sender_id);
        match deposited_token_opt {
            Some(deposited) => {
                self.deposited_token_1
                    .insert(sender_id, &(deposited + amount));
            }
            None => {
                self.deposited_token_1.insert(sender_id, &amount);
            }
        }
    }

    fn token_0_approve(&mut self, owner_id: &AccountId, account_id: &AccountId) {
        self.approved_token_0.insert(owner_id, account_id);
    }

    fn token_1_approve(&mut self, owner_id: &AccountId, account_id: &AccountId) {
        self.approved_token_1.insert(owner_id, account_id);
    }
}
