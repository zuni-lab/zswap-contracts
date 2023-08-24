use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise,
    PromiseError,
};
use std::cmp::Ordering;
use zswap_math_library::pool_account;

use error::*;
use pool::*;

mod error;
pub mod pool;

const NEAR_PER_STORAGE: Balance = 10_000_000_000_000_000_000; // 10e18yⓃ
const ZSWAP_POOL_CONTRACT: &[u8] = include_bytes!("../../res/zswap_pool.wasm");

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    fees: LookupMap<u32, u32>,
    pools: LookupMap<AccountId, bool>,
    // Since a contract is something big to store, we use LazyOptions
    // this way it is not deserialized on each method call
    code: LazyOption<Vec<u8>>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Fees,
    Pools,
    Code,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        let mut fees = LookupMap::new(StorageKey::Fees);
        fees.insert(&500, &10);
        fees.insert(&3000, &60);

        Self {
            fees,
            pools: LookupMap::new(StorageKey::Pools),
            code: LazyOption::new(StorageKey::Code, Some(&ZSWAP_POOL_CONTRACT.to_vec())),
        }
    }

    pub fn get_pool(&self, token_0: AccountId, token_1: AccountId, fee: u32) -> Option<PoolView> {
        let ordered_token_0;
        let ordered_token_1;
        match token_0.cmp(&token_1) {
            Ordering::Less => {
                ordered_token_0 = token_0;
                ordered_token_1 = token_1;
            }
            Ordering::Greater => {
                ordered_token_0 = token_1;
                ordered_token_1 = token_0;
            }
            Ordering::Equal => return None,
        }

        let pool_id = pool_account::compute_account(
            env::current_account_id(),
            ordered_token_0.clone(),
            ordered_token_1.clone(),
            fee,
        );

        if !self.pools.get(&pool_id).unwrap_or_default() {
            return None;
        }

        let pool_view = PoolView {
            pool_id,
            token_0: ordered_token_0,
            token_1: ordered_token_1,
            fee,
            tick_spacing: self.fees.get(&fee).unwrap(),
        };

        Some(pool_view)
    }

    #[payable]
    pub fn create_pool(&mut self, token_0: AccountId, token_1: AccountId, fee: u32) -> Promise {
        let tick_spacing_opt = self.fees.get(&fee);
        if tick_spacing_opt.is_none() {
            env::panic_str(UNSUPPORTED_FEE);
        }

        let ordered_token_0;
        let ordered_token_1;
        match token_0.cmp(&token_1) {
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

        let subaccount = pool_account::compute_account(
            env::current_account_id(),
            ordered_token_0.clone(),
            ordered_token_1.clone(),
            fee,
        );

        if !env::is_valid_account_id(subaccount.as_bytes()) {
            env::panic_str(INVALID_SUBACCOUNT);
        }

        if self.pools.get(&subaccount).unwrap_or_default() {
            env::panic_str(POOL_ALREADY_EXISTS);
        }

        self.pools.insert(&subaccount, &true);

        // Assert enough money is attached to create the account and deploy the contract
        let attached = env::attached_deposit();

        let code = self.code.get().unwrap();
        let contract_bytes = code.len() as u128;
        let minimum_needed = NEAR_PER_STORAGE * contract_bytes;

        if attached < minimum_needed {
            env::panic_str(&format!("Attach at least {} yⓃ", minimum_needed));
        }

        log!("Signer Public Key {:?}", env::signer_account_pk());

        let promise = Promise::new(subaccount.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(attached)
            .deploy_contract(code)
            .and(ext_zswap_pool::ext(subaccount.clone()).new(
                ordered_token_0,
                ordered_token_1,
                tick_spacing_opt.unwrap(),
                fee,
            ));

        // Add callback
        promise.then(
            Self::ext(env::current_account_id()).create_factory_subaccount_and_deploy_callback(
                subaccount,
                env::predecessor_account_id(),
                attached,
            ),
        )
    }

    #[private]
    pub fn create_factory_subaccount_and_deploy_callback(
        &mut self,
        account: AccountId,
        deployer: AccountId,
        attached: Balance,
        #[callback_result] create_deploy_result: Result<(), PromiseError>,
    ) -> Option<AccountId> {
        if let Ok(_result) = create_deploy_result {
            log!(format!("Correctly created and deployed to {account}"));
            return Some(account);
        };

        log!(format!(
            "Error creating {account}, returning {attached}yⓃ to {deployer}"
        ));
        Promise::new(deployer).transfer(attached);
        None
    }
}
