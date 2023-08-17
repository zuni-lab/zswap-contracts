use near_sdk::ext_contract;

#[ext_contract(ext_ft_zswap_manager)]
pub trait FtZswapManager {
    fn transfer_approved_tokens_to_mint(amount0: u128, amount1: u128, data: Vec<u8>);
    fn transfer_approved_tokens_to_swap(amount0: u128, amount1: u128, data: Vec<u8>);
}
