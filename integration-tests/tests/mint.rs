use near_sdk::{json_types::U128, ONE_YOCTO};
use serde_json::json;

use zswap_manager::utils::MintParams;

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
    let token_0_amount = U128::from(100);
    let token_1_amount = U128::from(500_000);
    println!("\nDepositing token 0 & 1 into ZswapManager...");
    liquidity_provider
        .call(context.token_0_contract.id(), "ft_transfer_call")
        .args_json((
            context.manager_contract.id(),
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
            context.manager_contract.id(),
            token_1_amount,
            None::<String>,
            String::from(""),
        ))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let deposited_token_0 = context
        .manager_contract
        .call("get_deposited_token")
        .args_json((liquidity_provider.id(), context.token_0_contract.id()))
        .view()
        .await?
        .json::<U128>()?;
    let deposited_token_1 = context
        .manager_contract
        .call("get_deposited_token")
        .args_json((liquidity_provider.id(), context.token_1_contract.id()))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(deposited_token_0, token_0_amount);
    assert_eq!(deposited_token_1, token_1_amount);
    println!("✅ Deposited token 0 & 1 into `ZswapManger`");

    let mint_params = MintParams {
        token_0: context.token_0_contract.id().parse().unwrap(),
        token_1: context.token_1_contract.id().parse().unwrap(),
        fee: POOL_FEE,
        lower_tick: -500,
        upper_tick: 500,
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

    let balance_token_0_pool = context
        .token_0_contract
        .call("ft_balance_of")
        .args_json(json!({"account_id": context.pool_id}))
        .view()
        .await?
        .json::<U128>()?;
    println!("balance_token_0_pool: {:?}", balance_token_0_pool);
    assert_eq!(balance_token_0_pool, added_amounts[0]);

    let balance_token_1_pool = context
        .token_1_contract
        .call("ft_balance_of")
        .args_json(json!({"account_id": context.pool_id}))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(balance_token_1_pool, added_amounts[1]);

    println!("✅ Minted liquidity tokens");

    Ok(())
}
