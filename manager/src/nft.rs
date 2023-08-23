use near_contract_standards::non_fungible_token::{
    metadata::{NFTContractMetadata, NonFungibleTokenMetadataProvider},
    Token, TokenId,
};
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
