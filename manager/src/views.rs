use near_sdk::{json_types::U128, near_bindgen, AccountId};

use crate::{utils::get_approved_token_key, Contract, ContractExt};

#[near_bindgen]
impl Contract {
    pub fn get_deposited_token(&self, owner: AccountId, token: AccountId) -> U128 {
        let owner_account = self.get_account(&owner);
        match owner_account.deposited_tokens.get(&token) {
            Some(deposited_amount) => deposited_amount.into(),
            None => 0.into(),
        }
    }

    pub fn get_approved_token(
        &self,
        owner: AccountId,
        spender: AccountId,
        token: AccountId,
    ) -> U128 {
        let owner_account = self.get_account(&owner);
        let approved_token_key = get_approved_token_key(&spender, &token);
        match owner_account.approved_tokens.get(&approved_token_key) {
            Some(approved_amount) => approved_amount.into(),
            None => 0.into(),
        }
    }
}
