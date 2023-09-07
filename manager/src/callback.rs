use near_contract_standards::non_fungible_token::metadata::TokenMetadata as NftMetadata;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{env, near_bindgen, serde_json, Promise, PromiseError};
use zswap_math_library::pool_account;

use crate::error::{MINT_CALLBACK_ERROR, SLIPPAGE_CHECK_FAILED};
use crate::nft::NftPosition;
use crate::utils::{generate_nft_media, MintCallbackParams, NftLiquidityInfo};
use crate::Contract;
use crate::ContractExt;

pub trait ManagerCallback {
    fn mint_callback(
        &mut self,
        used_amounts_res: Result<[U128; 2], PromiseError>,
        params: MintCallbackParams,
    ) -> [U128; 2];

    fn manager_swap_callback(
        &mut self,
        amount_out_min: U128,
        amount_received: Result<U128, PromiseError>,
    );
}

#[near_bindgen]
impl ManagerCallback for Contract {
    #[private]
    #[payable]
    fn mint_callback(
        &mut self,
        #[callback_result] used_amounts_res: Result<[U128; 2], PromiseError>,
        params: MintCallbackParams,
    ) -> [U128; 2] {
        if used_amounts_res.is_err() {
            Promise::new(params.recipient).transfer(env::attached_deposit());
            env::panic_str(MINT_CALLBACK_ERROR)
        }

        let received_amounts = used_amounts_res.unwrap();
        let amount_0 = received_amounts[0];
        let amount_1 = received_amounts[1];

        if amount_0 < params.amount_0_min || amount_1 < params.amount_1_min {
            Promise::new(params.recipient).transfer(env::attached_deposit());
            env::panic_str(SLIPPAGE_CHECK_FAILED)
        }

        // mint nft
        let symbol_0 = &params.symbol_0;
        let symbol_1 = &params.symbol_1;
        assert!(params.token_0 < params.token_1);
        let pool = pool_account::compute_account(
            &self.factory,
            &params.token_0,
            &params.token_1,
            params.fee,
        );

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
            liquidity: params.liquidity,
        };

        let liquidity_nft_metadata = NftMetadata {
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
            params.recipient,
            Some(liquidity_nft_metadata),
        );
        self.nft_positions.insert(
            &self.nft_id,
            &NftPosition {
                pool,
                lower_tick: params.lower_tick,
                upper_tick: params.upper_tick,
                liquidity: params.liquidity,
            },
        );
        self.nft_id += 1;

        [amount_0, amount_1]
    }

    #[allow(unused)]
    #[private]
    fn manager_swap_callback(
        &mut self,
        amount_out_min: U128,
        #[callback_result] amount_received: Result<U128, PromiseError>,
    ) {
        todo!("check output amount is enough or not")
    }
}
