use near_sdk::{env, AccountId};

pub fn compute_account(
    factory: AccountId,
    token_0: AccountId,
    token_1: AccountId,
    fee: u32,
) -> AccountId {
    let hash_data = env::keccak256(
        [token_0.as_bytes(), token_1.as_bytes(), &fee.to_le_bytes()]
            .concat()
            .as_slice(),
    );

    let subaccount: AccountId = format!("{}.{}", hex::encode(&hash_data[0..8]), factory)
        .parse()
        .unwrap();

    subaccount
}
