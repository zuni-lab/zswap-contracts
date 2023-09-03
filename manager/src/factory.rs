use near_sdk::{ext_contract, AccountId, Promise};

#[ext_contract(ext_zswap_factory)]
pub trait ZswapFactory {
    fn create_pool(&mut self, token_0: AccountId, token_1: AccountId, fee: u32) -> Promise;
}
