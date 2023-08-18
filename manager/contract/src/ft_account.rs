use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env;
use near_sdk::{collections::UnorderedMap, AccountId, Balance, StorageUsage};

use crate::error::EXCEED_ALLOWANCE;
use crate::StorageKey;

pub type TokenId = AccountId;

/// Account deposits information and storage cost.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    pub deposited_tokens: UnorderedMap<TokenId, Balance>, // token => amount
    pub approved_tokens: UnorderedMap<AccountId, UnorderedMap<TokenId, Balance>>, // spender => token => amount

    /// TODO: add storage usage, implement later
    pub near_amount: Balance,
    pub storage_used: StorageUsage,
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Self {
            deposited_tokens: UnorderedMap::new(StorageKey::AccountDepositedTokens {
                account_id: account_id.to_owned(),
            }),
            approved_tokens: UnorderedMap::new(StorageKey::AccountApprovedTokens {
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
        let mut token_approved = self
            .approved_tokens
            .get(spender_id)
            .unwrap_or(UnorderedMap::new(StorageKey::AccountApprovedSpender {
                spender_id: spender_id.to_owned(),
                token_id: token_id.to_owned(),
            }));
        token_approved.insert(token_id, &amount);

        self.approved_tokens.insert(spender_id, &token_approved);
    }

    // only overwrite if the amount is different
    pub fn internal_collect_approved_token(
        &mut self,
        spender_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) {
        let mut token_approved = self.approved_tokens.get(spender_id).unwrap();
        let approved_amount = token_approved.get(token_id).unwrap_or(0);

        if approved_amount < amount {
            env::panic_str(EXCEED_ALLOWANCE);
        }

        token_approved.insert(token_id, &(approved_amount - amount));

        self.approved_tokens.insert(spender_id, &token_approved);
    }
}
