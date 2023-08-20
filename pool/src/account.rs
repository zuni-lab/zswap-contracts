use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{collections::UnorderedMap, AccountId, Balance, StorageUsage};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    /// Native NEAR amount sent to the exchange.
    /// Used for storage right now, but in future can be used for trading as well.
    pub near_amount: Balance,
    /// Amounts of various tokens deposited to this account.
    pub tokens: UnorderedMap<AccountId, Balance>,
    pub storage_used: StorageUsage,
}
