use near_sdk::{json_types::U128, ONE_YOCTO};
use near_units::parse_near;
use serde_json::json;

use zswap_manager::{ft_receiver::TokenReceiverMessage, utils::MintParams};

use helper::*;

mod helper;

#[tokio::test]
async fn test_mint_properly() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    println!("\nContracts setup...");
    let context = init(&worker).await?;
    let liquidity_provider = context.deployer;
    println!("✅ Setup done");

    // deposit token 0 & 1 into deployer
    let token_0_amount = U128::from(100_000);
    let token_1_amount = U128::from(500_000);
    liquidity_provider
        .call(context.token_0_contract.id(), "ft_transfer_call")
        .args_json((
            context.pool_id.clone(),
            token_0_amount,
            None::<String>,
            String::from(""),
        ))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    liquidity_provider
        .call(context.token_1_contract.id(), "ft_transfer_call")
        .args_json((
            context.pool_id.clone(),
            token_1_amount,
            None::<String>,
            String::from(""),
        ))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    println!("✅ Deposited token 0 & 1 into `ZswapPool`");

    let mint_params = MintParams {
        token_0: context.token_0_contract.id().parse().unwrap(),
        token_1: context.token_1_contract.id().parse().unwrap(),
        fee: POOL_FEE,
        lower_tick: 46000,
        upper_tick: 46200,
        amount_0_desired: token_0_amount,
        amount_1_desired: token_1_amount,
        amount_0_min: U128::from(0),
        amount_1_min: U128::from(0),
    };
    liquidity_provider
        .call(context.manager_contract.id(), "mint")
        .args_json(json!({ "params": mint_params }))
        .deposit(parse_near!("0.1 N"))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    println!("✅ Minted liquidity tokens");

    // swap
    let token_0_balance_before_swap = liquidity_provider
        .call(context.token_0_contract.id(), "ft_balance_of")
        .args_json(json!({"account_id": liquidity_provider.id()}))
        .view()
        .await?
        .json::<U128>()?;

    let token_1_balance_before_swap = liquidity_provider
        .call(context.token_1_contract.id(), "ft_balance_of")
        .args_json(json!({"account_id": liquidity_provider.id()}))
        .view()
        .await?
        .json::<U128>()?;

    let msg = TokenReceiverMessage::SwapSingle {
        token_out: context.token_1_contract.id().parse().unwrap(),
        fee: POOL_FEE,
        sqrt_price_limit_x96: None,
    };
    liquidity_provider
        .call(context.token_0_contract.id(), "ft_transfer_call")
        .args_json((
            context.manager_contract.id(),
            U128::from(777),
            None::<String>,
            near_sdk::serde_json::to_string(&msg).unwrap(),
        ))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let token_0_balance_after_swap = liquidity_provider
        .call(context.token_0_contract.id(), "ft_balance_of")
        .args_json(json!({"account_id": liquidity_provider.id()}))
        .view()
        .await?
        .json::<U128>()?;
    let token_1_balance_after_swap = liquidity_provider
        .call(context.token_1_contract.id(), "ft_balance_of")
        .args_json(json!({"account_id": liquidity_provider.id()}))
        .view()
        .await?
        .json::<U128>()?;

    assert!(token_0_balance_before_swap.0 > token_0_balance_after_swap.0);
    assert!(token_1_balance_before_swap.0 < token_1_balance_after_swap.0);

    println!("✅ Swapped token 0 to token 1");

    Ok(())
}
