use near_sdk::{env, log, near_bindgen, AccountId, Promise};

use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn update_stored_contract(&mut self) {
        self.code
            .set(&env::input().expect("Error: No input").to_vec());
    }

    #[payable]
    #[private]
    pub fn redeploy_pool(&mut self, token_0: AccountId, token_1: AccountId, fee: u32) -> Promise {
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

        if !self.pools.get(&subaccount).unwrap_or_default() {
            env::panic_str("Pool does not exist");
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
}
