use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::ONE_YOCTO;
use near_sdk::{env, json_types::U128, near_bindgen, serde_json, AccountId, PromiseOrValue};
use zswap_math_library::pool_account;

use crate::error::*;
use crate::pool::ext_zswap_pool;
use crate::{Contract, ContractExt};

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    SwapSingle {
        token_out: AccountId,
        fee: u32,
        sqrt_price_limit_x96: Option<U128>,
    },
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        let amount_in = amount;

        let message = serde_json::from_str::<TokenReceiverMessage>(&msg).expect(WRONG_MSG_FORMAT);
        match message {
            TokenReceiverMessage::SwapSingle {
                token_out,
                fee,
                sqrt_price_limit_x96,
            } => {
                let receipient = sender_id;
                let zero_for_one = token_in < token_out;
                let pool_id =
                    pool_account::compute_account(&self.factory, &token_in, &token_out, fee);
                ext_ft_core::ext(token_in)
                    .with_attached_deposit(ONE_YOCTO)
                    .ft_transfer_call(pool_id.clone(), amount_in, None, String::from(""))
                    .then(ext_zswap_pool::ext(pool_id).swap(
                        receipient,
                        zero_for_one,
                        amount_in,
                        sqrt_price_limit_x96,
                    ));
            }
        }

        let unused_amount = U128(0);
        PromiseOrValue::Value(unused_amount)
    }
}

// impl Contract {
//     fn internal_deposit(&mut self, sender_id: &AccountId, token_id: &AccountId, amount: u128) {
//         let token_key = get_token_key(sender_id, token_id);
//         let token_opt = self.account_tokens.get(&token_key);
//         match token_opt {
//             Some(deposistted) => {
//                 self.account_tokens
//                     .insert(&token_key, &(deposistted + amount));
//             }
//             None => {
//                 self.account_tokens.insert(&token_key, &amount);
//             }
//         }
//     }
// }
