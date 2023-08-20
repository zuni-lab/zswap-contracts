use near_sdk::{serde::Serialize, AccountId};

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInitArgs {
    pub factory: AccountId,
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub tick_spacing: u32,
    pub fee: u32,
}
