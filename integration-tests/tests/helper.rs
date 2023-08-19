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
        factory_contract: factory_contract,
        pool_id: pool_id,
        manager_contract,
        deployer,
    })
}
