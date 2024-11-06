use std::{env, str::FromStr, sync::Arc};

use alloy::{primitives::Address, signers::Signature, transports::http::reqwest::Url};
use myrtle_wyckoff_dstack::{
    artifacts::IDepositRegistry,
    errors::MwError,
    gulper,
    jtrain::Jtrain,
    orderhere::{self, CancelOrder, Order},
    settler::create_settlement_order,
    snapshotter,
    structs::UserRequest,
};
use optimized_lob::order::OrderId;
use rocket::{
    catch, catchers, delete, get, http::Status, launch, post, put, response::Redirect, routes,
    serde::json::Json, tokio::sync::RwLock, Request, State,
};

struct AppState {
    jtrain: Jtrain,
}

type SharedState = Arc<RwLock<AppState>>;

#[catch(default)]
fn default_catcher(status: Status, request: &Request) -> String {
    format!("ERROR: {} - {:?}", status.code, status.reason())
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to("https://tplus.cx")
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, world!\n\nTODO description of t+?"
}

#[get("/health")]
fn health() -> &'static str {
    "Healthy!"
}

#[get("/public-key")]
async fn get_public_key(state: &State<SharedState>) -> String {
    let jtrain = &state.read().await.jtrain;
    jtrain.warehouse.signer.address().to_string()
}

#[put("/contract-addresses/<deposit_registry_address>/<checkpointer_address>")]
async fn set_contract_addresses(
    state: &State<SharedState>,
    deposit_registry_address: String,
    checkpointer_address: String,
) -> String {
    let mut guard = state.write().await;

    guard.jtrain.warehouse.deposit_contract = Address::from_str(&deposit_registry_address).unwrap();
    guard.jtrain.warehouse.checkpoint_contract = Address::from_str(&checkpointer_address).unwrap();
    guard.jtrain.warehouse.store();
    "Thanks!".to_string()
}

#[post("/new-settlement-order/<user>/<taker_signature>", data = "<order>")]
async fn new_settlement_order(
    state: &State<SharedState>,
    user: String,
    taker_signature: String,
    order: Json<IDepositRegistry::Order>,
) -> Result<String, MwError> {
    let jtrain = &state.read().await.jtrain;
    let user = Address::from_raw_public_key(user.as_bytes());
    let taker_signature = Signature::from_str(&taker_signature).unwrap();
    let new_order = create_settlement_order(
        &jtrain.warehouse,
        &jtrain.provider,
        user,
        order.0,
        taker_signature,
    )
    .await?;
    let mut guard = state.write().await;
    guard.jtrain.warehouse.add_settlement_order(new_order);
    Ok("Added settlement order. Thanks!".to_string())
}

#[get("/get-settlement-order-length")]
async fn get_settlement_order_length(state: &State<SharedState>) -> String {
    let jtrain = &state.read().await.jtrain;
    format!("{:?}", jtrain.warehouse.settlement_orders.len())
}

#[get("/get-orders/<user>/<signature>", data = "<request>")]
async fn get_orders(
    state: &State<SharedState>,
    user: String,
    signature: String,
    request: Json<UserRequest>,
) -> Result<String, MwError> {
    let jtrain = &state.read().await.jtrain;
    let user = Address::from_raw_public_key(user.as_bytes());
    let signature = Signature::from_str(&signature).unwrap();
    request.validate_signature(signature, user)?;
    request.validate_timestamp()?;
    request.validate_request_type("orders")?;
    let orders = jtrain
        .warehouse
        .get_orders(&jtrain.orderbook_manager, user)?;
    Ok(serde_json::to_string(&orders).unwrap())
}

#[post("/send-order/<user>/<signature>", data = "<order>")]
async fn send_order(
    state: &State<SharedState>,
    user: String,
    signature: String,
    order: Json<Order>,
) -> Result<String, MwError> {
    let mut guard = state.write().await;
    let user = Address::from_raw_public_key(user.as_bytes());
    let signature = Signature::from_str(&signature).unwrap();
    let jtrain = &mut guard.jtrain;
    let result = orderhere::new_order(
        &mut jtrain.warehouse,
        &mut jtrain.orderbook_manager,
        user,
        order.0,
        signature,
    )?;

    Ok(format!("{:?}", result))
}

#[delete("/cancel-order/<user>/<signature>", data = "<cancel>")]
async fn cancel_order(
    state: &State<SharedState>,
    user: String,
    signature: String,
    cancel: Json<CancelOrder>,
) -> Result<String, MwError> {
    let mut guard = state.write().await;
    let user = Address::from_raw_public_key(user.as_bytes());
    let signature = Signature::from_str(&signature).unwrap();
    let jtrain = &mut guard.jtrain;
    orderhere::cancel_order(
        user,
        cancel.0,
        signature,
        &mut jtrain.warehouse,
        &mut jtrain.orderbook_manager,
    )?;
    Ok("Thanks!".to_string())
}

#[put("/modify-order/<user>/<signature>/<order_id>", data = "<order>")]
async fn modify_order(
    state: &State<SharedState>,
    user: String,
    signature: String,
    order_id: String,
    order: Json<Order>,
) -> Result<String, MwError> {
    let mut guard = state.write().await;
    let user = Address::from_raw_public_key(user.as_bytes());
    let signature = Signature::from_str(&signature).unwrap();
    let jtrain = &mut guard.jtrain;
    let order_id = OrderId(u32::from_str(&order_id).unwrap());
    let new_oid = orderhere::replace_order(
        user,
        order.0,
        order_id,
        signature,
        &mut jtrain.warehouse,
        &mut jtrain.orderbook_manager,
    )?;
    Ok(format!("{:?}", new_oid))
}

#[get("/get-inventory/<user>/<signature>", data = "<request>")]
async fn get_inventory(
    state: &State<SharedState>,
    user: String,
    signature: String,
    request: Json<UserRequest>,
) -> Result<String, MwError> {
    let jtrain = &state.read().await.jtrain;
    let user = Address::from_raw_public_key(user.as_bytes());
    let signature = Signature::from_str(&signature).unwrap();
    request.validate_signature(signature, user)?;
    request.validate_timestamp()?;
    request.validate_request_type("inventory")?;
    let inventory = jtrain
        .warehouse
        .inventories
        .get(&user)
        .cloned()
        .unwrap_or_default();
    Ok(inventory.to_json())
}

#[put("/gulp-deposits/<user>")]
async fn gulp_deposits(state: &State<SharedState>, user: String) -> Result<String, MwError> {
    let mut guard = state.write().await;
    let jtrain = &mut guard.jtrain;
    let user = Address::from_raw_public_key(user.as_bytes());
    let new_deposits = gulper::gulp_deposits(&mut jtrain.warehouse, &jtrain.provider, user)
        .map_err(|e| MwError::GulpError(e.to_string()))?;
    Ok(format!("{:?}", new_deposits))
}

///@dev: run ever 5 seconds
#[post("/take_snapshot")]
async fn take_snapshot(state: &State<SharedState>) -> Result<String, MwError> {
    let jtrain = &state.read().await.jtrain;
    let tx_receipt = snapshotter::snapshot(&jtrain.warehouse, &jtrain.provider)
        .await
        .map_err(|e| MwError::SnapshotError(e.to_string()))?;

    Ok(serde_json::to_string(&tx_receipt)
        .unwrap_or_else(|_| "Failed to serialize transaction receipt".to_string()))
}

#[launch]
async fn rocket() -> _ {
    let initial_state = AppState {
        jtrain: Jtrain::new(Url::from_str(&env::var("RPC_URL").unwrap().to_string()).unwrap())
            .await,
    };
    let shared_state = Arc::new(RwLock::new(initial_state));

    rocket::build()
        .manage(shared_state)
        .mount(
            "/",
            routes![
                index,
                health,
                set_contract_addresses,
                get_public_key,
                hello,
                new_settlement_order,
                get_settlement_order_length,
                get_orders,
                send_order,
                cancel_order,
                modify_order,
                get_inventory,
                gulp_deposits,
                take_snapshot,
            ],
        )
        .register("/", catchers![default_catcher])
}
