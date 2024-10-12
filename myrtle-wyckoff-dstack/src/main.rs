use std::sync::Arc;

use myrtle_wyckoff_dstack::jtrain::Jtrain;
use rocket::{
    delete, get, launch, post, put, response::Redirect, routes, tokio::sync::RwLock, State,
};

struct AppState {
    jtrain: Jtrain,
}

type SharedState = Arc<RwLock<AppState>>;

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
    format!("{:?}", read_lock.jtrain.warehouse.books.len())
}

#[post("/create-settlement-order")]
async fn create_settlement_order(state: &State<SharedState>) -> String {
    todo!()
}

#[get("/get-orders")]
async fn get_orders(state: &State<SharedState>) -> String {
    todo!()
}

#[post("/send-order")]
async fn send_order(state: &State<SharedState>) -> String {
    todo!()
}

#[delete("/cancel-order")]
async fn cancel_order(state: &State<SharedState>) -> String {
    todo!()
}

#[put("/modify-order")]
async fn modify_order(state: &State<SharedState>) -> String {
    todo!()
}

#[get("/get-inventory")]
async fn get_inventory(state: &State<SharedState>) -> String {
    todo!()
}

#[post("/take-snapshot")]
async fn take_snapshot(state: &State<SharedState>) -> String {
    todo!()
}

#[launch]
fn rocket() -> _ {
    let initial_state = AppState {
        jtrain: Jtrain::new(),
    };
    let shared_state = Arc::new(RwLock::new(initial_state));

    rocket::build().manage(shared_state).mount(
        "/",
        routes![
            index,
            health,
            state,
            hello,
            create_settlement_order,
            get_orders,
            send_order,
            cancel_order,
            modify_order,
            get_inventory,
            take_snapshot
        ],
    )
}
