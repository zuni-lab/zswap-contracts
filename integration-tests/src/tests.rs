use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{json_types::U128, ONE_YOCTO};
use near_units::parse_near;
use serde_json::json;
use std::fs;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};
// use zswap_manager::utils::MintParams; // FIX: can not import?

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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Slot0 {
    // Current sqrt(P)
    pub sqrt_price_x96: U128,
    // Current tick
    pub tick: i32,
}

const FT_WASM_FILEPATH: &str = "../res/mock/fungible_token.wasm";
const POOL_WASM_FILEPATH: &str = "../res/zswap_pool.wasm";
const MANAGER_WASM_FILEPATH: &str = "../res/zswap_manager.wasm";

#[derive(Debug, Clone)]
struct TestContext {
    token_0_contract: Contract,
    token_1_contract: Contract,
    pool_contract: Contract,
    manager_contract: Contract,
    deployer: Account,
}

async fn init(worker: &Worker<impl DevNetwork>) -> anyhow::Result<TestContext> {
    let ft_wasm = fs::read(FT_WASM_FILEPATH)?;
    let pool_wasm = fs::read(POOL_WASM_FILEPATH)?;
    let manager_wasm = fs::read(MANAGER_WASM_FILEPATH)?;

    let token_0_contract = worker.dev_deploy(&ft_wasm).await?;
    let token_1_contract = worker.dev_deploy(&ft_wasm).await?;
    let pool_contract = worker.dev_deploy(&pool_wasm).await?;
    let manager_contract = worker.dev_deploy(&manager_wasm).await?;

    let account = worker.dev_create_account().await?;
    let deployer = account
        .create_subaccount("deployer")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .into_result()?;
    let factory = account
        .create_subaccount("factory")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;

    let initial_balance = U128::from(parse_near!("100 N"));
    token_0_contract
        .call("new_default_meta")
        .args_json((deployer.id(), initial_balance))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    token_1_contract
        .call("new_default_meta")
        .args_json((deployer.id(), initial_balance))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    pool_contract
        .call("new")
        .args_json((
            factory.id(),
            token_0_contract.id(),
            token_1_contract.id(),
            60,
            3000,
        ))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    manager_contract
        .call("new")
        .args_json((factory.id(), deployer.id()))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let slot_0 = pool_contract
        .call("get_slot_0")
        .view()
        .await?
        .json::<Slot0>()?;

    // add storage deposit for manager & pool contract
    deployer
        .call(token_0_contract.id(), "storage_deposit")
        .args_json((manager_contract.id(), None::<bool>))
        .deposit(parse_near!("1 N"))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    deployer
        .call(token_0_contract.id(), "storage_deposit")
        .args_json((pool_contract.id(), None::<bool>))
        .deposit(parse_near!("1 N"))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    deployer
        .call(token_1_contract.id(), "storage_deposit")
        .args_json((manager_contract.id(), None::<bool>))
        .deposit(parse_near!("1 N"))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    deployer
        .call(token_1_contract.id(), "storage_deposit")
        .args_json((pool_contract.id(), None::<bool>))
        .deposit(parse_near!("1 N"))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    Ok(TestContext {
        token_0_contract,
        token_1_contract,
        pool_contract,
        manager_contract,
        deployer,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    println!("Setupping...");
    let context = init(&worker).await?;
    let liqudity_provider = context.deployer;
    println!("✅ Setup done");

    // deposit token 0 & 1 into deployer
    let token_0_amount = U128::from(100);
    let token_1_amount = U128::from(500_000);
    println!("Depositing token 0 & 1 into deployer...");
    liqudity_provider
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
    liqudity_provider
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
        .args_json((liqudity_provider.id(), context.token_0_contract.id()))
        .view()
        .await?
        .json::<U128>()?;
    let deposited_token_1 = context
        .manager_contract
        .call("get_deposited_token")
        .args_json((liqudity_provider.id(), context.token_1_contract.id()))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(deposited_token_0, token_0_amount);
    assert_eq!(deposited_token_1, token_1_amount);
    println!("✅ Deposited token 0 & 1 into `ZswapManger`");

    let mint_params = MintParams {
        token_0: context.token_0_contract.id().to_owned(),
        token_1: context.token_1_contract.id().to_owned(),
        fee: 3000,
        lower_tick: -500,
        upper_tick: 500,
        amount_0_desired: token_0_amount,
        amount_1_desired: token_1_amount,
        amount_0_min: U128::from(50),
        amount_1_min: U128::from(100_000),
    };
    let res = liqudity_provider
        .call(context.manager_contract.id(), "mint")
        .args_json(json!({ "params": mint_params }))
        .max_gas()
        .transact()
        .await?;
    res.logs().iter().for_each(|log| {
        println!("log: {:?}", log);
    });
    res.into_result()?;

    Ok(())
}

// #[tokio::test]
// async fn mint_liqudity() -> anyhow::Result<()> {
//     let worker = workspaces::sandbox().await?;
//     let context = init(&worker).await?;
//     let liqudity_provider = context.deployer;

//     // deposit token 0 & 1 into deployer
//     liqudity_provider
//         .call(context.token_0_contract.id(), "ft_transfer_and_call")
//         .args_json((
//             context.manager_contract.id(),
//             U128::from(parse_near!("1 N")),
//             None::<String>,
//         ))
//         .transact()
//         .await?
//         .into_result()?;
//     liqudity_provider
//         .call(context.token_1_contract.id(), "ft_transfer_and_call")
//         .args_json((
//             context.manager_contract.id(),
//             U128::from(parse_near!("1 N")),
//             None::<String>,
//         ))
//         .transact()
//         .await?
//         .into_result()?;

//     let deposited_token_0 = context
//         .manager_contract
//         .call("get_deposited_token")
//         .args_json((liqudity_provider.id(), context.token_0_contract.id()))
//         .view()
//         .await?
//         .json::<U128>()?;
//     println!("deposited_token_0: {}", deposited_token_0.0);

//     liqudity_provider
//         .call(context.manager_contract.id(), "mint_liquidity")
//         .args_json((
//             context.pool_contract.id(),
//             context.token_0_contract.id(),
//             context.token_1_contract.id(),
//             U128::from(parse_near!("1 N")),
//             U128::from(parse_near!("1 N")),
//             60,
//             3000,
//         ))
//         .transact()
//         .await?
//         .into_result()?;

//     Ok(())
// }

// async fn test_default_message(user: &Account, contract: &Contract) -> anyhow::Result<()> {
//     let message: String = user
//         .call(contract.id(), "get_greeting")
//         .args_json(json!({}))
//         .transact()
//         .await?
//         .json()?;

//     assert_eq!(message, "Hello".to_string());
//     println!("      Passed ✅ gets default message");
//     Ok(())
// }

// async fn test_changes_message(user: &Account, contract: &Contract) -> anyhow::Result<()> {
//     user.call(contract.id(), "set_greeting")
//         .args_json(json!({"message": "Howdy"}))
//         .transact()
//         .await?
//         .into_result()?;

//     let message: String = user
//         .call(contract.id(), "get_greeting")
//         .args_json(json!({}))
//         .transact()
//         .await?
//         .json()?;

//     assert_eq!(message, "Howdy".to_string());
//     println!("      Passed ✅ changes message");
//     Ok(())
// }
