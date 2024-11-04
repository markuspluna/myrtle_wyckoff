use std::{env, str::FromStr, sync::Arc};

use alloy::{primitives::Address, signers::Signature, transports::http::reqwest::Url};
use myrtle_wyckoff_dstack::{
    artifacts::IDepositRegistry,
    gulper,
    jtrain::Jtrain,
    orderhere::{self, CancelOrder, Order},
    settler::create_settlement_order,
    snapshotter, warehouse,
};
use optimized_lob::order::OrderId;
use rocket::{
    catch, catchers, delete,
    fairing::{Fairing, Info, Kind},
    get,
    http::Status,
    launch, post, put,
    response::{Redirect, Responder, Result},
    routes,
    serde::json::Json,
    tokio::sync::RwLock,
    Error, Request, State,
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
fn index(state: &State<SharedState>) -> Redirect {
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

#[get("/state")]
async fn state(state: &State<SharedState>) -> String {
    let read_lock = state.read().await;
    format!("{:?}", read_lock.jtrain.orderbook_manager.books.len())
}

#[post("/new-settlement-order/<user>/<taker_signature>", data = "<order>")]
async fn new_settlement_order(
    state: &State<SharedState>,
    user: String,
    taker_signature: String,
    order: Json<IDepositRegistry::Order>,
) -> String {
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
    .await;
    let mut guard = state.write().await;
    guard.jtrain.warehouse.add_settlement_order(new_order);
    "Thanks!".to_string()
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
) -> String {
    let jtrain = &state.read().await.jtrain;
    let user = Address::from_raw_public_key(user.as_bytes());
    request.validate_signature(signature, user);
    request.validate_timestamp();
    request.validate_request_type("orders");
    let orders = jtrain.warehouse.get_orders(&jtrain.orderbook_manager, user);
    serde_json::to_string(&orders).unwrap()
}

#[post("/send-order/<user>/<signature>", data = "<order>")]
async fn send_order(
    state: &State<SharedState>,
    user: String,
    signature: String,
    order: Json<Order>,
) -> String {
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
    );

    format!("{:?}", result)
}

#[delete("/cancel-order/<user>/<signature>", data = "<cancel>")]
async fn cancel_order(
    state: &State<SharedState>,
    user: String,
    signature: String,
    cancel: Json<CancelOrder>,
) -> String {
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
    );
    "Thanks!".to_string()
}

#[put("/modify-order/<user>/<signature>/<order_id>", data = "<order>")]
async fn modify_order(
    state: &State<SharedState>,
    user: String,
    signature: String,
    order_id: String,
    order: Json<Order>,
) -> String {
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
    );
    format!("{:?}", new_oid)
}

#[get("/get-inventory/<user>/<signature>", data = "<request>")]
async fn get_inventory(
    state: &State<SharedState>,
    user: String,
    signature: String,
    request: Json<UserRequest>,
) -> String {
    let jtrain = &state.read().await.jtrain;
    let user = Address::from_raw_public_key(user.as_bytes());
    let signature = Signature::from_str(&signature).unwrap();
    request.validate_signature(signature, user);
    request.validate_timestamp();
    request.validate_request_type("inventory");
    jtrain.warehouse.inventories.get(&user).unwrap().to_json()
}

#[put("/gulp-deposits/<user>")]
async fn gulp_deposits(state: &State<SharedState>, user: String) -> String {
    let mut guard = state.write().await;
    let jtrain = &mut guard.jtrain;
    let user = Address::from_raw_public_key(user.as_bytes());
    gulper::gulp_deposits(&mut jtrain.warehouse, &jtrain.provider, user);
    "Thanks!".to_string()
}

#[post("/take_snapshot")]
async fn take_snapshot(state: &State<SharedState>) -> String {
    let jtrain = &state.read().await.jtrain;
    snapshotter::snapshot(&jtrain.warehouse, &jtrain.provider);
    "Thanks!".to_string()
}

#[launch]
fn rocket() -> _ {
    let initial_state = AppState {
        jtrain: Jtrain::new(Url::from_str(&env::var("RPC_URL").unwrap().to_string()).unwrap()),
    };
    let shared_state = Arc::new(RwLock::new(initial_state));

    rocket::build()
        .manage(shared_state)
        .mount(
            "/",
            routes![
                index,
                health,
                state,
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
