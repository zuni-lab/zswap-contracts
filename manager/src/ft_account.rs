use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{collections::LookupMap, AccountId, Balance, StorageUsage};
use near_sdk::{env, CryptoHash};

use crate::error::EXCEED_ALLOWANCE;
use crate::utils::get_approved_token_key;
use crate::StorageKey;

pub type TokenId = AccountId;

/// Account deposits information and storage cost.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    pub deposited_tokens: LookupMap<TokenId, Balance>, // token => amount
    pub approved_tokens: LookupMap<CryptoHash, Balance>, // spender => token => amount

    /// TODO: add storage usage, implement later
    pub near_amount: Balance,
    pub storage_used: StorageUsage,
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Self {
            deposited_tokens: LookupMap::new(StorageKey::AccountDepositedTokens {
                account_id: account_id.to_owned(),
            }),
            approved_tokens: LookupMap::new(StorageKey::AccountApprovedTokens {
                account_id: account_id.to_owned(),
            }),
            near_amount: 0,
            storage_used: 0,
        }
    }

    // only overwrite if the amount is different
    pub fn internal_approve_token(
        &mut self,
        spender_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) {
        let approved_token_key = get_approved_token_key(spender_id, token_id);
        self.approved_tokens.insert(&approved_token_key, &amount);
    }

    // only overwrite if the amount is different
    pub fn internal_collect_and_reset_approved_token(
        &mut self,
        spender_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) {
        let approved_token_key = get_approved_token_key(spender_id, token_id);
        let approved_amount = self.approved_tokens.get(&approved_token_key).unwrap();
        let depositted_amount = self.deposited_tokens.get(token_id).unwrap_or(0);

        if amount != 0 && (approved_amount > depositted_amount || approved_amount < amount) {
            env::panic_str(EXCEED_ALLOWANCE);
        }

        self.approved_tokens.remove(&approved_token_key);
        self.deposited_tokens
            .insert(token_id, &(depositted_amount - amount));
    }
}
