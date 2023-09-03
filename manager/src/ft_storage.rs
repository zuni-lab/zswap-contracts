use near_contract_standards::storage_management::StorageBalance;
use near_sdk::{ext_contract, AccountId};

#[ext_contract(ext_ft_storage)]
pub trait StorageManagement {
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;
}
