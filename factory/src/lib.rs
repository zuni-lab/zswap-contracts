use std::cmp::Ordering;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, BorshStorageKey, Gas, Promise, PromiseError,
};

use error::*;
use pool::PoolInitArgs;

mod error;
mod pool;

const NEAR_PER_STORAGE: Balance = 10_000_000_000_000_000_000; // 10e18yⓃ
const ZSWAP_POOL_CONTRACT: &[u8] = include_bytes!("../../res/zswap_pool.wasm");
const TGAS: Gas = Gas(10u64.pow(12)); // 10e12yⓃ
const NO_DEPOSIT: Balance = 0; // 0yⓃ

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    fees: LookupMap<u32, u32>,
    pools: LookupMap<Pool, AccountId>,
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

impl Default for Contract {
    fn default() -> Self {
        let mut fees = LookupMap::new(StorageKey::Fees);
        fees.insert(&500, &10);
        fees.insert(&3000, &60);

        Self {
            fees,
            pools: LookupMap::new(StorageKey::Pools),
            code: LazyOption::new(StorageKey::Code, Some(&ZSWAP_POOL_CONTRACT.to_vec())),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_pool(&self, token_0: AccountId, token_1: AccountId, fee: u32) -> Option<AccountId> {
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

        self.pools.get(&Pool {
            token_0: ordered_token_0,
            token_1: ordered_token_1,
            fee,
        })
    }

    #[payable]
    pub fn create_pool(&mut self, token_0: AccountId, token_1: AccountId, fee: u32) -> Promise {
        if self.fees.get(&fee) == None {
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

        let pool = Pool {
            token_0: ordered_token_0.clone(),
            token_1: ordered_token_1.clone(),
            fee,
        };

        if self.pools.get(&pool) != None {
            env::panic_str(POOL_ALREADY_EXISTS);
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
        let subaccount: AccountId = format!(
            "{}.{}",
            hex::encode(&hash_data[0..8]),
            env::current_account_id()
        )
        .parse()
        .unwrap();

        if !env::is_valid_account_id(subaccount.as_bytes()) {
            env::panic_str(INVALID_SUBACCOUNT);
        }
        self.pools.insert(&pool, &subaccount);

        // Assert enough money is attached to create the account and deploy the contract
        let attached = env::attached_deposit();

        let code = self.code.get().unwrap();
        let contract_bytes = code.len() as u128;
        let minimum_needed = NEAR_PER_STORAGE * contract_bytes;
        assert!(
            attached >= minimum_needed,
            "Attach at least {minimum_needed} yⓃ"
        );

        let init_args = near_sdk::serde_json::to_vec(&PoolInitArgs {
            factory: env::current_account_id(),
            token_0: ordered_token_0,
            token_1: ordered_token_1,
            tick_spacing: self.fees.get(&fee).unwrap(),
            fee,
        })
        .unwrap();

        let promise = Promise::new(subaccount.clone())
            .create_account()
            .transfer(attached)
            .deploy_contract(code)
            .function_call("new".to_owned(), init_args, NO_DEPOSIT, TGAS * 5);

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
