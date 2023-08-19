use near_sdk::{json_types::U128, ONE_YOCTO};
use near_units::parse_near;
use serde_json::json;
use std::fs;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

use zswap_manager::utils::MintParams;

const FT_WASM_FILEPATH: &str = "../res/mock/fungible_token.wasm";
const FACTORY_WASM_FILEPATH: &str = "../res/zswap_factory.wasm";
const MANAGER_WASM_FILEPATH: &str = "../res/zswap_manager.wasm";
const POOL_FEE: u32 = 3000;

#[derive(Debug, Clone)]
struct TestContext {
    token_0_contract: Contract,
    token_1_contract: Contract,
    _factory_contract: Contract,
    manager_contract: Contract,
    _pool_id: AccountId,
    deployer: Account,
}

async fn init(worker: &Worker<impl DevNetwork>) -> anyhow::Result<TestContext> {
    let ft_wasm = fs::read(FT_WASM_FILEPATH)?;
    let factory_wasm = fs::read(FACTORY_WASM_FILEPATH)?;
    let manager_wasm = fs::read(MANAGER_WASM_FILEPATH)?;

    let token_0_contract = worker.dev_deploy(&ft_wasm).await?;
    let token_1_contract = worker.dev_deploy(&ft_wasm).await?;
    let factory_contract = worker.dev_deploy(&factory_wasm).await?;
    let manager_contract = worker.dev_deploy(&manager_wasm).await?;

    let account = worker.dev_create_account().await?;
    let deployer = account
        .create_subaccount("deployer")
        .initial_balance(parse_near!("50 N"))
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

    manager_contract
        .call("new")
        .args_json(json!({"factory": factory_contract.id()}))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    // create new pool
    let pool_id = deployer
        .call(factory_contract.id(), "create_pool")
        .args_json((token_0_contract.id(), token_1_contract.id(), POOL_FEE))
        .deposit(parse_near!("10 N"))
        .max_gas()
        .transact()
        .await?
        .json::<AccountId>()?;

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
        .args_json((&pool_id, None::<bool>))
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
        .args_json((&pool_id, None::<bool>))
        .deposit(parse_near!("1 N"))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    Ok(TestContext {
        token_0_contract,
        token_1_contract,
        _factory_contract: factory_contract,
        _pool_id: pool_id,
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
        token_0: context.token_0_contract.id().parse().unwrap(),
        token_1: context.token_1_contract.id().parse().unwrap(),
        fee: POOL_FEE,
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
    println!("{:#?}", res.clone().into_result()?);
    res.logs().iter().for_each(|log| {
        println!("integration test log: {:?}", log);
    });

    Ok(())
}
