use near_sdk::{json_types::U128, ONE_YOCTO};
use serde_json::json;

use zswap_manager::utils::MintParams;

use helper::*;
use zswap_pool::utils::Slot0;

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
    // let deposited_token_0 = context
    //     .manager_contract
    //     .call("get_deposited_token")
    //     .args_json((liquidity_provider.id(), context.token_0_contract.id()))
    //     .view()
    //     .await?
    //     .json::<U128>()?;
    // let deposited_token_1 = context
    //     .manager_contract
    //     .call("get_deposited_token")
    //     .args_json((liquidity_provider.id(), context.token_1_contract.id()))
    //     .view()
    //     .await?
    //     .json::<U128>()?;
    // assert_eq!(deposited_token_0, token_0_amount);
    // assert_eq!(deposited_token_1, token_1_amount);
    println!("✅ Deposited token 0 & 1 into `ZswapPool`");

    let before_slot_0 = liquidity_provider
        .call(&context.pool_id, "get_slot_0")
        .view()
        .await?
        .json::<Slot0>()?;

    let mint_params = MintParams {
        token_0: context.token_0_contract.id().parse().unwrap(),
        token_1: context.token_1_contract.id().parse().unwrap(),
        fee: POOL_FEE,
        lower_tick: 46000,
        upper_tick: 46200,
        amount_0_desired: token_0_amount,
        amount_1_desired: token_0_amount,
        amount_0_min: U128::from(0),
        amount_1_min: U128::from(0),
    };

    let added_amounts = liquidity_provider
        .call(context.manager_contract.id(), "mint")
        .args_json(json!({ "params": mint_params }))
        .max_gas()
        .transact()
        .await?
        .json::<[U128; 2]>()?;
    println!("\tMinted amount 0: {}", added_amounts[0].0);
    println!("\tMinted amount 1: {}", added_amounts[1].0);

    let after_slot_0 = liquidity_provider
        .call(&context.pool_id, "get_slot_0")
        .view()
        .await?
        .json::<Slot0>()?;
    assert_eq!(before_slot_0.tick, after_slot_0.tick);
    assert_eq!(before_slot_0.sqrt_price_x96, after_slot_0.sqrt_price_x96);

    println!("✅ Minted liquidity tokens");

    Ok(())
}
