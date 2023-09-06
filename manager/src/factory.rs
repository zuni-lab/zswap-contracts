use near_sdk::{ext_contract, json_types::U128, AccountId, Promise};

#[ext_contract(ext_zswap_factory)]
pub trait ZswapFactory {
    fn create_pool(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        fee: u32,
        sqrt_price_x96: U128,
    ) -> Promise;
}
