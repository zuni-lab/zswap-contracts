use near_sdk::{ext_contract, serde::Serialize, AccountId};

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInitArgs {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub tick_spacing: u32,
    pub fee: u32,
}

#[ext_contract(ext_zswap_pool)]
pub trait FtZswapPool {
    fn new(token_0: AccountId, token_1: AccountId, tick_spacing: u32, fee: u32) -> Self;
}
