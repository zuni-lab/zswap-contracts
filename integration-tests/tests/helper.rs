use near_sdk::json_types::U128;
use near_units::parse_near;
use serde_json::json;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

const FT_CONTRACT: &[u8] = include_bytes!("../../res/mock/fungible_token.wasm");
const ZSWAP_MANAGER_CONTRACT: &[u8] = include_bytes!("../../res/zswap_manager.wasm");
const ZSWAP_FACTORY_CONTRACT: &[u8] = include_bytes!("../../res/zswap_factory.wasm");

pub const POOL_FEE: u32 = 3000;

#[derive(Debug, Clone)]
pub struct TestContext {
    pub token_0_contract: Contract,
    pub token_1_contract: Contract,
    pub factory_contract: Contract,
    pub manager_contract: Contract,
    pub pool_id: AccountId,
    pub deployer: Account,
}

pub async fn init(worker: &Worker<impl DevNetwork>) -> anyhow::Result<TestContext> {
    let token_0_contract = worker.dev_deploy(&FT_CONTRACT).await?;
    let token_1_contract = worker.dev_deploy(&FT_CONTRACT).await?;
    let factory_contract = worker.dev_deploy(&ZSWAP_FACTORY_CONTRACT).await?;
    let manager_contract = worker.dev_deploy(&ZSWAP_MANAGER_CONTRACT).await?;

    let support_fee_account = worker.dev_create_account().await?;
    let deployer = worker.dev_create_account().await?;
    support_fee_account
        .transfer_near(&deployer.id(), parse_near!("95 N"))
        .await?
        .into_result()?;
    println!("\tDeployer account {}", deployer.id());

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
    println!(
        "\tCreated token 0 {} & 1 {}",
        token_0_contract.id(),
        token_1_contract.id()
    );

    factory_contract
        .call("new")
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    println!("\tCreated factory {}", factory_contract.id());

    let pool_id = deployer
        .call(factory_contract.id(), "create_pool")
        .args_json((token_0_contract.id(), token_1_contract.id(), POOL_FEE))
        .deposit(parse_near!("30 N"))
        .max_gas()
        .transact()
        .await?
        .json::<AccountId>()?;
    println!("\tCreated pool {}", pool_id);

    let initial_sqrt_price_x96 = U128::from(10 * (2_u128).pow(96));
    deployer
        .call(&pool_id, "initialize")
        .args_json(json!({ "sqrt_price_x96": initial_sqrt_price_x96 }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    println!("\tInitialized with token_1/token_0: 100");

    manager_contract
        .call("new")
        .args_json(json!({"factory": factory_contract.id()}))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    println!("\tCreated manager {}", manager_contract.id());

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
        factory_contract,
        pool_id,
        manager_contract,
        deployer,
    })
}
