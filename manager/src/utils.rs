use base64::engine::general_purpose;
use base64::Engine;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, CryptoHash};

pub struct GetPositionParams {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub owner: AccountId,
    pub lower_tick: i32,
    pub upper_tick: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MintParams {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub amount_0_desired: U128,
    pub amount_1_desired: U128,
    pub amount_0_min: U128,
    pub amount_1_min: U128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapSingleParams {
    pub token_in: AccountId,
    pub token_out: AccountId,
    pub fee: u32,
    pub amount_in: U128,
    pub sqrt_price_limit_x96: Option<U128>,
}

#[allow(unused)]
pub struct SwapParams {
    tokens: Vec<AccountId>,
    fees: Vec<u32>,
    recipient: AccountId,
    amount_in: u128,
    amount_out_min: u128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapCallbackData {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub payer: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NftLiquidityInfo {
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub fee: u32,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub liquidity: u128,
}

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(crate = "near_sdk::serde")]
// pub struct PoolCallbackData {
//     pub token_0: AccountId,
//     pub token_1: AccountId,
//     pub payer: AccountId,
// }

pub fn get_token_key(owner: &AccountId, token_id: &AccountId) -> CryptoHash {
    env::keccak256_array(&[owner.as_bytes(), token_id.as_bytes()].concat())
}

pub fn generate_nft_media(
    symbol_0: &str,
    symbol_1: &str,
    owner: &AccountId,
    lower_tick: i32,
    upper_tick: i32,
    fee: u32,
) -> String {
    let image = [
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 300 480'>",
        "<style>.tokens { font: bold 30px sans-serif; }",
        ".fee { font: normal 26px sans-serif; }",
        ".tick { font: normal 18px sans-serif; }</style>",
        &render_background(owner, lower_tick, upper_tick),
        &render_top(symbol_0, symbol_1, fee),
        &render_bottom(lower_tick, upper_tick),
        "</svg>",
    ]
    .join("");

    let image_base64 = general_purpose::STANDARD.encode(image);

    ["data:image/svg+xml;base64,", &image_base64].join("")
}

fn render_background(owner: &AccountId, lower_tick: i32, upper_tick: i32) -> String {
    let key = env::keccak256_array(
        &[
            owner.as_bytes(),
            &lower_tick.to_le_bytes(),
            &upper_tick.to_le_bytes(),
        ]
        .concat(),
    );
    let hue = (u128::from_le_bytes((&key[0..16]).try_into().unwrap()) % 360).to_string();

    [
        "<rect width=\"300\" height=\"480\" fill=\"hsl(",
        &hue,
        ",40%,40%)\"/>",
        "<rect x=\"30\" y=\"30\" width=\"240\" height=\"420\" rx=\"15\" ry=\"15\" fill=\"hsl(",
        &hue,
        ",100%,50%)\" stroke=\"#000\"/>",
    ]
    .join("")
}
fn render_top(symbol_0: &str, symbol_1: &str, fee: u32) -> String {
    [
        "<rect x=\"30\" y=\"87\" width=\"240\" height=\"42\"/>",
        "<text x=\"39\" y=\"120\" class=\"tokens\" fill=\"#fff\">",
        symbol_0,
        "/",
        symbol_1,
        "</text>",
        "<rect x=\"30\" y=\"132\" width=\"240\" height=\"30\"/>",
        "<text x=\"39\" y=\"120\" dy=\"36\" class=\"fee\" fill=\"#fff\">",
        &fee.to_string(),
        "</text>",
    ]
    .join("")
}
fn render_bottom(lower_tick: i32, upper_tick: i32) -> String {
    [
        "<rect x=\"30\" y=\"342\" width=\"240\" height=\"24\"/>",
        "<text x=\"39\" y=\"360\" class=\"tick\" fill=\"#fff\">Lower tick: ",
        &lower_tick.to_string(),
        "</text>",
        "<rect x=\"30\" y=\"372\" width=\"240\" height=\"24\"/>",
        "<text x=\"39\" y=\"360\" dy=\"30\" class=\"tick\" fill=\"#fff\">Upper tick: ",
        &upper_tick.to_string(),
        "</text>",
    ]
    .join("")
}
