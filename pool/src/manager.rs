use near_sdk::{ext_contract, json_types::U128};

#[ext_contract(ext_ft_zswap_manager)]
pub trait FtZswapManager {
    fn collect_approved_tokens_to_mint(amount0: U128, amount1: U128, data: Vec<u8>);
    fn collect_approved_tokens_to_swap(amount0: U128, amount1: U128, data: Vec<u8>);
}
