use alloy::{primitives::Address, signers::Signature, sol, sol_types::SolStruct};
use optimized_lob::{
    order::OrderId, orderbook_manager::OrderBookManager, quantity::Qty, utils::BookId,
};
use tracing::info;

use crate::{
    domains::DSTACK_DOMAIN, jtrain::Jtrain, matchmaker::match_order, warehouse::Warehouse,
};

sol! {
    struct Order {
        uint256 price;
        uint256 qty;
        bool is_bid;
    }
}

pub fn new_order(
    warehouse: &mut Warehouse,
    orderbook_manager: &mut OrderBookManager,
    user: Address,
    order: Order,
    signature: Signature,
) {
    // validate signature
    let order_hash = order.eip712_signing_hash(&DSTACK_DOMAIN);
    if user != signature.recover_address_from_prehash(&order_hash).unwrap() {
        panic!("invalid signature")
    };
    // validate inventory state
    let user_inventory = warehouse.inventories.entry(user).or_default();

    // we don't need to validate taker inventory state since they're on margin
    if !user_inventory.is_taker {
        if order.is_bid && (order.qty * order.price) > user_inventory.usdc_balance.into_raw() {
            panic!("user does not have enough usdc");
        } else if order.qty > user_inventory.eth_balance.into_raw() {
            panic!("user does not have enough eth");
        }
    }
    let (qty_executed, volume_executed, matched_orders, new_order_id) = match_order(
        orderbook_manager,
        BookId(0),
        order.price,
        Qty(order.qty),
        order.is_bid,
    );
    info!(
        "new order: {:?}",
        (user, order.price, order.qty, order.is_bid)
    );
    info!("qty_executed: {}", qty_executed.0);
    info!("volume_executed: {}", volume_executed.0);
    info!("matched_orders: {:?}", matched_orders);
    info!("new_order_id: {:?}", new_order_id);
}

pub fn cancel_order(user: Address, oid: OrderId, jtrain: &mut Jtrain) {
    let order_exists = jtrain.warehouse.orders.get_mut(&user).unwrap().remove(&oid);
    if !order_exists {
        panic!("order does not exist")
    };
    jtrain.orderbook_manager.remove_order(oid);
}

pub fn modify_order() {}
