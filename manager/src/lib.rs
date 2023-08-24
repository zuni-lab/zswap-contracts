use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{I128, U128};
use near_sdk::{
    env, near_bindgen, serde_json, AccountId, BorshStorageKey, CryptoHash, PanicOnDefault, Promise,
    PromiseError,
};
use pool::{ext_zswap_pool, Slot0};
use zswap_math_library::num160::AsU160;
use zswap_math_library::num256::U256;
use zswap_math_library::{liquidity_math, tick_math};

use crate::utils::*;

mod callback;
mod error;
pub mod ft_receiver;
mod internal;
mod nft;
mod pool;
pub mod utils;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    factory: AccountId,
    token_id: u128,
    nft: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    account_tokens: LookupMap<CryptoHash, u128>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    AccountTokens,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(factory: AccountId) -> Self {
        let nft = NonFungibleToken::new(
            StorageKey::NonFungibleToken,
            env::current_account_id(),
            Some(StorageKey::TokenMetadata),
            Some(StorageKey::Enumeration),
            Some(StorageKey::Approval),
        );
        let metadata = NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name: "ZSwap Liquidity Management".to_string(),
            symbol: "ZLM".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        };
        Self {
            factory,
            token_id: 0,
            nft,
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            account_tokens: LookupMap::new(StorageKey::AccountTokens),
        }
    }

    #[allow(unused)]
    pub fn get_position(
        &self,
        token_1: AccountId,
        token_2: AccountId,
        fee: u32,
        owner: AccountId,
        lower_tick: i32,
        upper_tick: i32,
    ) {
    }

    #[payable]
    pub fn mint(&mut self, params: MintParams) -> Promise {
        let pool = self.get_pool(&params.token_0, &params.token_1, params.fee);
        let receipient = env::predecessor_account_id();

        ext_zswap_pool::ext(pool.clone()).get_slot_0().then(
            Self::ext(env::current_account_id())
                .with_attached_deposit(env::attached_deposit())
                .calculate_liquidity(pool, receipient, params),
        )
    }

    // #[payable]
    // pub fn swap_single(&mut self, params: SwapSingleParams) -> Promise {
    //     let receipient = env::predecessor_account_id();
    //     let token_in_key = get_token_key(&receipient, &params.token_in);
    //     let depositted_token_in = self
    //         .depositted_tokens
    //         .get(&token_in_key)
    //         .unwrap_or_default();
    //     if depositted_token_in < params.amount_in.0 {
    //         env::panic_str(INSUFFICIENT_FUND);
    //     }

    //     let zero_for_one = params.token_in < params.token_out;
    //     let pool_id = pool_account::compute_account(
    //         self.factory.clone(),
    //         params.token_in.clone(),
    //         params.token_out.clone(),
    //         params.fee,
    //     );
    //     ext_ft_core::ext(params.token_in.clone())
    //         .with_attached_deposit(ONE_YOCTO)
    //         .ft_transfer_call(pool_id.clone(), params.amount_in, None, String::from(""))
    //         .then(ext_zswap_pool::ext(pool_id).swap(
    //             receipient,
    //             zero_for_one,
    //             params.amount_in,
    //             params.sqrt_price_limit_x96,
    //         ))
    //     // .then(Self::ext(env::current_account_id()).handle_multi_swap_calback())
    // }

    // #[payable]
    // pub fn handle_multi_swap_calback(&mut self) {
    //     log!("promise count: {}", env::promise_results_count());
    //     log!("promise result: {:?}", env::promise_result(0));
    //     log!("calling handle_multi_swap_calback")
    // }

    // #[allow(unused)]
    // #[payable]
    // pub fn swap(
    //     &mut self,
    //     tokens: Vec<AccountId>,
    //     fees: Vec<u32>,
    //     recipient: AccountId,
    //     amount_in: u128,
    //     amount_out_min: u128,
    // ) {
    // }

    #[payable]
    #[private]
    pub fn calculate_liquidity(
        &mut self,
        #[callback_result] slot_0_res: Result<Slot0, PromiseError>,
        pool: AccountId,
        recipient: AccountId,
        params: MintParams,
    ) -> Promise {
        let slot_0 = slot_0_res.unwrap();

        let sqrt_price_x96 = slot_0.sqrt_price_x96;
        let sqrt_price_lower_x96 = tick_math::get_sqrt_ratio_at_tick(params.lower_tick);
        let sqrt_price_upper_x96 = tick_math::get_sqrt_ratio_at_tick(params.upper_tick);
        let liquidity = liquidity_math::get_liquidity_for_amounts(
            U256::from(sqrt_price_x96.0),
            sqrt_price_lower_x96.as_u160(),
            sqrt_price_upper_x96.as_u160(),
            U256::from(params.amount_0_desired.0),
            U256::from(params.amount_1_desired.0),
        );

        // mint nft
        let liquidity_info = NftLiquidityInfo {
            token_0: params.token_0.clone(),
            token_1: params.token_1.clone(),
            fee: params.fee,
            lower_tick: params.lower_tick,
            upper_tick: params.upper_tick,
            liquidity,
        };
        let nft_description = format!("ZSwap Liquidity NFT for {}", &pool);

        let liqudity_nft_metadata = TokenMetadata {
            title: Some("ZSwap Liquidity NFT".to_string()),
            description: Some(nft_description),
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: Some(serde_json::to_string(&liquidity_info).unwrap()),
            reference: None,
            reference_hash: None,
        };

        self.nft.internal_mint(
            self.token_id.to_string(),
            recipient.clone(),
            Some(liqudity_nft_metadata),
        );
        self.token_id += 1;

        ext_zswap_pool::ext(pool)
            .mint(
                recipient,
                params.lower_tick,
                params.upper_tick,
                U128::from(liquidity),
            )
            .then(
                Self::ext(env::current_account_id())
                    .mint_callback(params.amount_0_min.into(), params.amount_1_min.into()),
            )
    }

    #[private]
    pub fn calculate_amount_out(
        &mut self,
        #[callback_result] amounts_res: Result<[I128; 2], PromiseError>,
        zero_for_one: bool,
    ) -> U128 {
        let amounts = amounts_res.unwrap();
        if zero_for_one {
            let amount_1 = -amounts[1].0 as u128;
            U128::from(amount_1)
        } else {
            let amount_0 = -amounts[0].0 as u128;
            U128::from(amount_0)
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {}
