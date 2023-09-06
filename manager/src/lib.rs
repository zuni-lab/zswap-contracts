use near_contract_standards::fungible_token::metadata::{ext_ft_metadata, FungibleTokenMetadata};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, I128, U128};
use near_sdk::{
    env, log, near_bindgen, serde_json, AccountId, Balance, BorshStorageKey, CryptoHash,
    PanicOnDefault, Promise, PromiseError,
};
use zswap_math_library::num256::U256;
use zswap_math_library::{liquidity_math, sqrt_price_math, tick_math};

use crate::error::*;
use crate::factory::ext_zswap_factory;
use crate::ft_storage::ext_ft_storage;
use crate::nft::*;
use crate::pool::{ext_zswap_pool, Slot0};
use crate::utils::*;

mod callback;
mod error;
mod factory;
pub mod ft_receiver;
mod ft_storage;
mod internal;
mod nft;
mod pool;
pub mod utils;

const NEAR_PER_STORAGE: Balance = 10_000_000_000_000_000_000; // 10e18yⓃ
const FT_STORAGE_DEPOSIT: Balance = 1500 * NEAR_PER_STORAGE;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    factory: AccountId,
    account_tokens: LookupMap<CryptoHash, u128>,
    fungible_tokens: UnorderedSet<AccountId>,
    nft_positions: LookupMap<u128, NftPosition>,
    nft: NonFungibleToken,
    nft_id: u128,
    metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    AccountTokens,
    FungibleTokens,
    NftPositions,
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
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
            account_tokens: LookupMap::new(StorageKey::AccountTokens),
            fungible_tokens: UnorderedSet::new(StorageKey::FungibleTokens),
            nft_positions: LookupMap::new(StorageKey::NftPositions),
            nft,
            nft_id: 0,
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    #[payable]
    pub fn create_pool(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        fee: u32,
        sqrt_price_x96: U128,
    ) -> Promise {
        let is_new_token_0 = self.fungible_tokens.insert(&token_0);
        let is_new_token_1 = self.fungible_tokens.insert(&token_1);
        if !is_new_token_0 && !is_new_token_1 {
            env::panic_str(POOL_ALREADY_EXISTS);
        }

        let create_pool_promise = ext_zswap_factory::ext(self.factory.clone())
            .with_attached_deposit(env::attached_deposit() - 2 * FT_STORAGE_DEPOSIT)
            .with_unused_gas_weight(30)
            .create_pool(token_0.clone(), token_1.clone(), fee, sqrt_price_x96);

        let token_0_storage_deposit_promise = ext_ft_storage::ext(token_0.clone())
            .with_attached_deposit(FT_STORAGE_DEPOSIT)
            .storage_deposit(Some(env::current_account_id()), None);
        let token_1_storage_deposit_promise = ext_ft_storage::ext(token_1.clone())
            .with_attached_deposit(FT_STORAGE_DEPOSIT)
            .storage_deposit(Some(env::current_account_id()), None);

        if is_new_token_0 && is_new_token_1 {
            token_0_storage_deposit_promise
                .and(token_1_storage_deposit_promise)
                .then(create_pool_promise)
        } else if is_new_token_0 {
            token_0_storage_deposit_promise.then(create_pool_promise)
        } else {
            token_1_storage_deposit_promise.then(create_pool_promise)
        }
    }

    #[payable]
    pub fn mint(&mut self, params: MintParams) -> Promise {
        let pool = self.internal_get_pool(&params.token_0, &params.token_1, params.fee);
        let slot_0_promise = ext_zswap_pool::ext(pool.clone()).get_slot_0();

        let token_0_meta_promise = ext_ft_metadata::ext(params.token_0.clone()).ft_metadata();
        let token_1_meta_promise = ext_ft_metadata::ext(params.token_1.clone()).ft_metadata();

        let recipient = env::predecessor_account_id();

        slot_0_promise
            .and(token_0_meta_promise)
            .and(token_1_meta_promise)
            .then(
                Self::ext(env::current_account_id())
                    .with_attached_deposit(env::attached_deposit())
                    .mint_calculate_liquidity(pool, recipient, params),
            )
    }

    #[payable]
    #[private]
    pub fn mint_calculate_liquidity(
        &mut self,
        #[callback_result] slot_0_res: Result<Slot0, PromiseError>,
        #[callback_result] token_0_meta_res: Result<FungibleTokenMetadata, PromiseError>,
        #[callback_result] token_1_meta_res: Result<FungibleTokenMetadata, PromiseError>,
        pool: AccountId,
        recipient: AccountId,
        params: MintParams,
    ) -> Promise {
        let slot_0 = slot_0_res.unwrap();
        let liquidity = self.internal_calculate_liquidity(
            slot_0,
            params.lower_tick,
            params.upper_tick,
            params.amount_0_desired.0,
            params.amount_1_desired.0,
        );
        log!("Liquidity: {}", liquidity);

        // mint nft
        let token_0_meta = token_0_meta_res.unwrap();
        let token_1_meta = token_1_meta_res.unwrap();

        let symbol_0 = &token_0_meta.symbol;
        let symbol_1 = &token_1_meta.symbol;

        let nft_title = format!("{}/{}", symbol_0, symbol_1);
        let nft_description = format!("ZSwap Liquidity NFT for {}", &pool);
        let nft_media = generate_nft_media(
            self.nft_id,
            symbol_0,
            symbol_1,
            params.lower_tick,
            params.upper_tick,
            params.fee,
        );
        let nft_media_hash = env::sha256(nft_media.as_bytes());
        let liquidity_info = NftLiquidityInfo {
            token_0: params.token_0.clone(),
            token_1: params.token_1.clone(),
            fee: params.fee,
            lower_tick: params.lower_tick,
            upper_tick: params.upper_tick,
            liquidity,
        };

        let liquidity_nft_metadata = TokenMetadata {
            title: Some(nft_title),
            description: Some(nft_description),
            media: Some(nft_media),
            media_hash: Some(Base64VecU8::from(nft_media_hash)),
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
            self.nft_id.to_string(),
            recipient.clone(),
            Some(liquidity_nft_metadata),
        );
        self.nft_positions.insert(
            &self.nft_id,
            &NftPosition {
                pool: pool.clone(),
                lower_tick: params.lower_tick,
                upper_tick: params.upper_tick,
                liquidity,
            },
        );
        self.nft_id += 1;

        ext_zswap_pool::ext(pool)
            .mint(
                recipient,
                env::current_account_id(), // manager owns liquidity, recipient owns NFT
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

    #[payable]
    pub fn burn(&mut self, nft_id: U128) -> Promise {
        let nft_position = self.nft_positions.get(&nft_id.0);
        if nft_position.is_none() {
            env::panic_str(NFT_NOT_FOUND);
        }

        let nft_position = nft_position.unwrap();
        let owner = self.nft.owner_by_id.get(&nft_id.0.to_string()).unwrap();
        if owner != env::predecessor_account_id() {
            env::panic_str(NFT_NOT_OWNED_BY_CALLER);
        }

        ext_zswap_pool::ext(nft_position.pool.clone())
            .burn(
                nft_position.lower_tick,
                nft_position.upper_tick,
                nft_position.liquidity.into(),
            )
            .then(Self::ext(env::current_account_id()).burn_callback(
                nft_position.pool,
                owner,
                nft_id,
                nft_position.lower_tick,
                nft_position.upper_tick,
            ))
    }

    #[private]
    pub fn burn_callback(
        &mut self,
        #[callback_result] token_amounts_res: Result<[U128; 2], PromiseError>,
        pool: AccountId,
        recipient: AccountId,
        nft_id: U128,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Promise {
        self.nft.internal_burn(nft_id.0.to_string(), &recipient);
        self.nft_positions.remove(&nft_id.0);

        log!("Burned NFT {:?}", nft_id);

        let token_amounts = token_amounts_res.unwrap();

        ext_zswap_pool::ext(pool).collect(
            recipient,
            lower_tick,
            upper_tick,
            token_amounts[0],
            token_amounts[1],
        )
    }

    pub fn get_liquidity_for_amounts(
        &self,
        slot_0: Slot0,
        lower_tick: i32,
        upper_tick: i32,
        amount_0_desired: U128,
        amount_1_desired: U128,
    ) -> U128 {
        self.internal_calculate_liquidity(
            slot_0,
            lower_tick,
            upper_tick,
            amount_0_desired.0,
            amount_1_desired.0,
        )
        .into()
    }

    pub fn calculate_amount_1_with_amount_0(
        &self,
        amount_0: U128,
        sqrt_price_x96: U128,
        lower_tick: i32,
        upper_tick: i32,
    ) -> U128 {
        let sqrt_price_x96 = U256::from(sqrt_price_x96.0);
        let sqrt_price_lower_x96 = tick_math::get_sqrt_ratio_at_tick(lower_tick);
        let sqrt_price_upper_x96 = tick_math::get_sqrt_ratio_at_tick(upper_tick);

        if !(sqrt_price_lower_x96..=sqrt_price_upper_x96).contains(&sqrt_price_x96) {
            return U128::from(0);
        }

        let liquidity = liquidity_math::get_liquidity_for_amount_0(
            sqrt_price_x96,
            sqrt_price_upper_x96,
            amount_0.0,
        );

        sqrt_price_math::get_amount_1_delta_signed(
            sqrt_price_lower_x96,
            sqrt_price_x96,
            liquidity as i128,
        )
        .abs()
        .as_u128()
        .into()
    }

    pub fn calculate_amount_0_with_amount_1(
        &self,
        amount_1: U128,
        sqrt_price_x96: U128,
        lower_tick: i32,
        upper_tick: i32,
    ) -> U128 {
        let sqrt_price_x96 = U256::from(sqrt_price_x96.0);
        let sqrt_price_lower_x96 = tick_math::get_sqrt_ratio_at_tick(lower_tick);
        let sqrt_price_upper_x96 = tick_math::get_sqrt_ratio_at_tick(upper_tick);

        if !(sqrt_price_lower_x96..=sqrt_price_upper_x96).contains(&sqrt_price_x96) {
            return U128::from(0);
        }

        let liquidity = liquidity_math::get_liquidity_for_amount_1(
            sqrt_price_lower_x96,
            sqrt_price_x96,
            amount_1.0,
        );

        sqrt_price_math::get_amount_0_delta_signed(
            sqrt_price_x96,
            sqrt_price_upper_x96,
            liquidity as i128,
        )
        .abs()
        .as_u128()
        .into()
    }

    pub fn get_fungible_tokens(&self) -> Vec<AccountId> {
        self.fungible_tokens.to_vec()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::{test_utils::VMContextBuilder, testing_env};

    #[test]
    fn test_calculate_amount_0_and_amount_1() {
        let factory_id = AccountId::new_unchecked("factory.testnet".to_string());
        testing_env!(VMContextBuilder::new().build());

        let contract = Contract::new(factory_id);
        let sqrt_price_x96 = U128::from(10 * (2_u128).pow(96));
        let lower_tick = 42000;
        let upper_tick = 48000;
        let amount_0 = U128::from(505327);
        let amount_1 = U128::from(100_000_111);

        let calculated_amount_1 = contract.calculate_amount_1_with_amount_0(
            amount_0,
            sqrt_price_x96,
            lower_tick,
            upper_tick,
        );

        assert_eq!(calculated_amount_1, amount_1);

        let calculated_amount_0 = contract.calculate_amount_0_with_amount_1(
            amount_1,
            sqrt_price_x96,
            lower_tick,
            upper_tick,
        );

        assert_eq!(calculated_amount_0, amount_0);
    }
}
