use ethnum::AsU256;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::non_fungible_token::NonFungibleToken;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{I128, U128};
use near_sdk::{
    env, log, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseError,
};
use pool::{ext_zswap_pool, Slot0};
use utils::{SwapCallbackData, SwapSingleParams};
use zswap_math_library::liquidity_math;
use zswap_math_library::num160::AsU160;
use zswap_math_library::tick_math::{self};

use crate::ft_account::Account;
use crate::utils::*;

mod callback;
mod error;
mod ft_account;
mod ft_receiver;
mod internal;
// mod nft;
mod pool;
pub mod utils;
mod views;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    factory: AccountId,
    nft: NonFungibleToken,
    accounts: LookupMap<AccountId, Account>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    AccountDepositedTokens {
        account_id: AccountId,
    },
    AccountApprovedTokens {
        account_id: AccountId,
    },
    AccountApprovedSpender {
        spender_id: AccountId,
        token_id: AccountId,
    },
    NonFungibleToken,
    // Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(factory: AccountId) -> Self {
        let nft = NonFungibleToken::new(
            StorageKey::NonFungibleToken,
            env::current_account_id(),
            Some(StorageKey::TokenMetadata),
            Some(StorageKey::Enumeration),
            Some(StorageKey::Approval),
        );
        Self {
            nft,
            factory,
            accounts: LookupMap::new(StorageKey::Accounts),
        }
    }

    #[allow(unused)]
    pub fn get_position(
        &self,
        token_1: AccountId,
        token_2: AccountId,
        fee: u32,
        owner: AccountId,
        lower_tick: i32,
        upper_tick: i32,
    ) {
    }

    #[payable]
    pub fn mint(&mut self, params: MintParams) -> Promise {
        let pool = self.get_pool(&params.token_0, &params.token_1, params.fee);
        let receipient = env::predecessor_account_id();

        ext_zswap_pool::ext(pool.clone()).get_slot_0().then(
            Self::ext(env::current_account_id()).calculate_liquidity(pool, receipient, params),
        )
    }

    #[payable]
    pub fn swap_single(&mut self, params: SwapSingleParams) -> Promise {
        let data = SwapCallbackData {
            token_0: params.token_in,
            token_1: params.token_out,
            fee: params.fee,
            payer: env::predecessor_account_id(),
        };

        self.internal_swap(
            params.amount_in,
            env::predecessor_account_id(),
            params.sqrt_price_limit_x96,
            data,
        )
    }

    #[allow(unused)]
    #[payable]
    pub fn swap(
        &mut self,
        tokens: Vec<AccountId>,
        fees: Vec<u32>,
        recipient: AccountId,
        amount_in: u128,
        amount_out_min: u128,
    ) {
    }

    #[payable]
    pub fn collect_approved_tokens_to_mint(
        &mut self,
        amount_0: U128,
        amount_1: U128,
        data: Vec<u8>,
    ) -> Promise {
        log!("manager/src/lib.rs line 112");
        let pool_callback_data: PoolCallbackData = near_sdk::serde_json::from_slice(&data).unwrap();

        let token_0 = pool_callback_data.token_0;
        let token_1 = pool_callback_data.token_1;
        let payer = pool_callback_data.payer;

        let mut payer_account = self.get_account(&payer);
        payer_account.internal_collect_approved_token(&payer, &token_0, amount_0.0);
        payer_account.internal_collect_approved_token(&payer, &token_1, amount_1.0);

        let transfer_token_0_promise =
            ext_ft_core::ext(token_0).ft_transfer(env::predecessor_account_id(), amount_0, None);
        let transfer_token_1_promise =
            ext_ft_core::ext(token_1).ft_transfer(env::predecessor_account_id(), amount_1, None);
        transfer_token_0_promise.and(transfer_token_1_promise)
    }

    #[private]
    pub fn calculate_liquidity(
        &mut self,
        #[callback_result] slot_0_res: Result<Slot0, PromiseError>,
        pool: AccountId,
        recipient: AccountId,
        params: MintParams,
    ) -> Promise {
        let slot_0 = slot_0_res.unwrap();
        let sqrt_price_x96 = slot_0.sqrt_price_x96;
        let sqrt_price_lower_x96 = tick_math::get_sqrt_ratio_at_tick(params.lower_tick);
        let sqrt_price_upper_x96 = tick_math::get_sqrt_ratio_at_tick(params.upper_tick);
        let liquidity = 0;
        // let liquidity = liquidity_math::get_liquidity_for_amounts(
        //     sqrt_price_x96.0.as_u256().as_u160(),
        //     sqrt_price_lower_x96.as_u160(),
        //     sqrt_price_upper_x96.as_u160(),
        //     params.amount_0_desired.0.as_u256(),
        //     params.amount_1_desired.0.as_u256(),
        // );
        // log!("manager/src/lib.rs line 144 `liquidity`: {}", liquidity);

        let pool_callback_data = PoolCallbackData {
            token_0: params.token_0.clone(),
            token_1: params.token_1.clone(),
            payer: recipient.clone(),
        };
        let data = near_sdk::serde_json::to_vec(&pool_callback_data).unwrap();

        let mut recipient_account = self.get_account(&recipient);
        recipient_account.internal_approve_token(&pool, &params.token_0, params.amount_0_desired.0);
        recipient_account.internal_approve_token(&pool, &params.token_1, params.amount_1_desired.0);

        ext_zswap_pool::ext(pool)
            .mint(
                recipient,
                params.lower_tick,
                params.upper_tick,
                U128::from(liquidity),
                data,
            )
            .then(
                Self::ext(env::current_account_id())
                    .manager_mint_callback(params.amount_0_min.0, params.amount_1_min.0),
            )
    }

    #[private]
    pub fn calculate_amount_out(
        &mut self,
        #[callback_result] amounts_res: Result<[I128; 2], PromiseError>,
        zero_for_one: bool,
    ) -> U128 {
        let amounts = amounts_res.unwrap();
        if zero_for_one {
            let amount_1 = -amounts[1].0 as u128;
            U128::from(amount_1)
        } else {
            let amount_0 = -amounts[0].0 as u128;
            U128::from(amount_0)
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn get_default_greeting() {
    //     let contract = Contract::default();
    //     // this test did not call set_greeting so should return the default "Hello" greeting
    //     assert_eq!(contract.get_greeting(), "Hello".to_string());
    // }

    // #[test]
    // fn set_then_get_greeting() {
    //     let mut contract = Contract::default();
    //     contract.set_greeting("howdy".to_string());
    //     assert_eq!(contract.get_greeting(), "howdy".to_string());
    // }
}
