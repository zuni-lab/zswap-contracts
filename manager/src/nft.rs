use near_contract_standards::non_fungible_token::{
    metadata::{NFTContractMetadata, NonFungibleTokenMetadataProvider},
    NonFungibleToken, Token, TokenId,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, Promise, PromiseOrValue};

use crate::{Contract, ContractExt};

near_contract_standards::impl_non_fungible_token_core!(Contract, nft);
near_contract_standards::impl_non_fungible_token_approval!(Contract, nft);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, nft);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NftPosition {
    pub pool: AccountId,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub liquidity: u128,
}

pub trait BurnNFT {
    fn internal_burn(&mut self, nft_id: String, nft_owner: &AccountId);
}

impl BurnNFT for NonFungibleToken {
    fn internal_burn(&mut self, nft_id: String, nft_owner: &AccountId) {
        if let Some(next_approval_id_by_id) = &mut self.next_approval_id_by_id {
            next_approval_id_by_id.remove(&nft_id);
        }
        if let Some(approvals_by_id) = &mut self.approvals_by_id {
            approvals_by_id.remove(&nft_id);
        }
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            let mut u = tokens_per_owner.remove(nft_owner).unwrap();
            u.remove(&nft_id);
        }
        if let Some(token_metadata_by_id) = &mut self.token_metadata_by_id {
            token_metadata_by_id.remove(&nft_id);
        }
        self.owner_by_id.remove(&nft_id);
    }
}
