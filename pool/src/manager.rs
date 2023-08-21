use near_sdk::{ext_contract, json_types::U128, Promise};

#[ext_contract(ext_ft_zswap_manager)]
pub trait FtZswapManager {
    fn collect_approved_tokens_to_mint(amount_0: U128, amount_1: U128, data: Vec<u8>) -> Promise;
    fn collect_approved_tokens_to_swap(amount_0: U128, amount_1: U128, data: Vec<u8>);
}
