use alloy::{
    primitives::{Address, U256},
    signers::{LocalWallet, Signature, Signer},
};
use myrtle_wyckoff_dstack::{
    jtrain::Jtrain,
    orderhere::{self, Order},
    settler::create_settlement_order,
    snapshotter,
    warehouse::Warehouse,
};
use optimized_lob::quantity::Qty;
use rocket::local::blocking::Client;
use serial_test::serial;

struct TestContext {
    rocket_client: Client,
    user_wallet: LocalWallet,
    taker_wallet: LocalWallet,
    deposit_registry: Address,
    checkpointer: Address,
}

impl TestContext {
    async fn setup() -> Self {
        // Start anvil in background
        std::process::Command::new("anvil")
            .arg("--port=8545")
            .spawn()
            .expect("Failed to start anvil");

        // Deploy contracts using Forge
        std::process::Command::new("forge")
            .args([
                "script",
                "script/Deploy.s.sol",
                "--broadcast",
                "--rpc-url",
                "http://localhost:8545",
            ])
            .output()
            .expect("Failed to deploy contracts");

        // Get deployed addresses from deployment artifacts
        let deployment = std::fs::read_to_string("broadcast/Deploy.s.sol/31337/run-latest.json")
            .expect("Failed to read deployment file");
        let json: serde_json::Value = serde_json::from_str(&deployment).unwrap();

        let deposit_registry =
            Address::from_str(&json["transactions"][0]["contractAddress"].as_str().unwrap())
                .unwrap();
        let checkpointer =
            Address::from_str(&json["transactions"][1]["contractAddress"].as_str().unwrap())
                .unwrap();

        // Create test wallets
        let user_wallet = LocalWallet::random();
        let taker_wallet = LocalWallet::random();

        // Build rocket instance
        let rocket = rocket::build()
            .mount(
                "/",
                routes![
                    index,
                    hello,
                    health,
                    get_public_key,
                    set_contract_addresses,
                    new_settlement_order,
                    get_settlement_order_length,
                    get_orders,
                    send_order,
                    cancel_order,
                    modify_order
                ],
            )
            .manage(Jtrain::new("http://localhost:8545".parse().unwrap()).await);

        Self {
            rocket_client: Client::tracked(rocket).expect("Failed to create rocket client"),
            user_wallet,
            taker_wallet,
            deposit_registry,
            checkpointer,
        }
    }
}

#[tokio::test]
#[serial]
async fn test_full_order_flow() {
    let ctx = TestContext::setup().await;

    // 1. Set contract addresses
    let response = ctx
        .rocket_client
        .put(format!(
            "/contract-addresses/{}/{}",
            ctx.deposit_registry, ctx.checkpointer
        ))
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    // 2. User places order
    let order = Order {
        price: U256::from(1500),
        qty: U256::from(100_000_000),
        is_bid: false,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let signature = ctx.user_wallet.sign_typed_data(&order).await.unwrap();

    let response = ctx
        .rocket_client
        .post(format!(
            "/send-order/{}/{}",
            ctx.user_wallet.address(),
            signature
        ))
        .json(&order)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    // 3. Taker matches order
    let taker_order = Order {
        price: U256::from(1500),
        qty: U256::from(100_000_000),
        is_bid: true,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let taker_signature = ctx
        .taker_wallet
        .sign_typed_data(&taker_order)
        .await
        .unwrap();

    let response = ctx
        .rocket_client
        .post(format!(
            "/new-settlement-order/{}/{}",
            ctx.taker_wallet.address(),
            taker_signature
        ))
        .json(&taker_order)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    // 4. Verify settlement order was created
    let response = ctx
        .rocket_client
        .get("/get-settlement-order-length")
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(response.into_string().unwrap(), "1");
}

#[tokio::test]
#[serial]
async fn test_cancel_order() {
    let ctx = TestContext::setup().await;

    // Setup contract addresses
    let response = ctx
        .rocket_client
        .put(format!(
            "/contract-addresses/{}/{}",
            ctx.deposit_registry, ctx.checkpointer
        ))
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    // Place order
    let order = Order {
        price: U256::from(1500),
        qty: U256::from(100_000_000),
        is_bid: false,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let signature = ctx.user_wallet.sign_typed_data(&order).await.unwrap();
    let response = ctx
        .rocket_client
        .post(format!(
            "/send-order/{}/{}",
            ctx.user_wallet.address(),
            signature
        ))
        .json(&order)
        .dispatch();

    let order_id = response.into_string().unwrap();

    // Cancel order
    let cancel_order = CancelOrder {
        order_id: OrderId(order_id.parse().unwrap()),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let cancel_signature = ctx
        .user_wallet
        .sign_typed_data(&cancel_order)
        .await
        .unwrap();

    let response = ctx
        .rocket_client
        .delete(format!(
            "/cancel-order/{}/{}",
            ctx.user_wallet.address(),
            cancel_signature
        ))
        .json(&cancel_order)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
}

#[tokio::test]
#[serial]
async fn test_modify_order() {
    let ctx = TestContext::setup().await;

    // Setup initial order
    let order = Order {
        price: U256::from(1500),
        qty: U256::from(100_000_000),
        is_bid: false,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let signature = ctx.user_wallet.sign_typed_data(&order).await.unwrap();
    let response = ctx
        .rocket_client
        .post(format!(
            "/send-order/{}/{}",
            ctx.user_wallet.address(),
            signature
        ))
        .json(&order)
        .dispatch();

    let order_id = response.into_string().unwrap();

    // Modify order
    let modified_order = Order {
        price: U256::from(1600),     // New price
        qty: U256::from(50_000_000), // New quantity
        is_bid: false,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let modify_signature = ctx
        .user_wallet
        .sign_typed_data(&modified_order)
        .await
        .unwrap();

    let response = ctx
        .rocket_client
        .put(format!(
            "/modify-order/{}/{}/{}",
            ctx.user_wallet.address(),
            modify_signature,
            order_id
        ))
        .json(&modified_order)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
}

#[tokio::test]
#[serial]
async fn test_gulp_deposits() {
    let ctx = TestContext::setup().await;

    // Setup deposits in the contract first using Forge script
    std::process::Command::new("forge")
        .args([
            "script",
            "script/SetupDeposits.s.sol",
            "--sig",
            "run(address)",
            "--rpc-url",
            "http://localhost:8545",
            "--broadcast",
            &format!("{}", ctx.user_wallet.address()),
        ])
        .output()
        .expect("Failed to setup deposits");

    // Gulp deposits
    let response = ctx
        .rocket_client
        .put(format!("/gulp-deposits/{}", ctx.user_wallet.address()))
        .dispatch();

    assert_eq!(response.status(), rocket::http::Status::Ok);

    // Verify deposits were gulped by checking inventory
    let request = UserRequest {
        request_type: "inventory".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let signature = ctx.user_wallet.sign_typed_data(&request).await.unwrap();

    let response = ctx
        .rocket_client
        .get(format!(
            "/get-inventory/{}/{}",
            ctx.user_wallet.address(),
            signature
        ))
        .json(&request)
        .dispatch();

    let inventory: Inventory = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(inventory.eth_balance > Qty(U256::ZERO));
}

#[tokio::test]
#[serial]
async fn test_settlement_and_snapshot() {
    let ctx = TestContext::setup().await;

    // Place maker order
    let maker_order = Order {
        price: U256::from(1500),
        qty: U256::from(100_000_000),
        is_bid: false,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    let maker_signature = ctx.user_wallet.sign_typed_data(&maker_order).await.unwrap();

    ctx.rocket_client
        .post(format!(
            "/send-order/{}/{}",
            ctx.user_wallet.address(),
            maker_signature
        ))
        .json(&maker_order)
        .dispatch();

    // Create settlement order
    let taker_order = IDepositRegistry::Order {
        ethAmount: U256::from(100_000_000),
        usdcAmount: U256::from(150_000_000_000),
        isBid: true,
        nonce: U256::ZERO,
    };

    let taker_signature = ctx
        .taker_wallet
        .sign_typed_data(&taker_order)
        .await
        .unwrap();

    let response = ctx
        .rocket_client
        .post(format!(
            "/new-settlement-order/{}/{}",
            ctx.taker_wallet.address(),
            taker_signature
        ))
        .json(&taker_order)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    // Take snapshot
    let response = ctx.rocket_client.post("/take_snapshot").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    // Verify snapshot was recorded on chain
    let checkpointer = ICheckpointer::new(ctx.checkpointer, &ctx.provider);
    let nonce = checkpointer
        .inventory_checkpoint_nonce()
        .call()
        .await
        .unwrap();
    assert_eq!(nonce, U256::from(1));
}

// Add more test cases for other endpoints...
