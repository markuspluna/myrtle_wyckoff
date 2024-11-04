use crate::{domains::DSTACK_DOMAIN, matchmaker::match_order, warehouse::Warehouse};
use alloy::{
    primitives::{Address, U256},
    signers::Signature,
    sol,
    sol_types::SolStruct,
};
use core::ops::{AddAssign as AddAssignTrait, SubAssign as SubAssignTrait};
use optimized_lob::{
    order::OrderId, orderbook_manager::OrderBookManager, price::Price, quantity::Qty, utils::BookId,
};
use tracing::info;
// Note: We're not going to use strongreplay protection for now as theoretically the TEE should handle that
sol! {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Order {
        uint256 price;
        uint256 qty;
        bool is_bid;
        uint64 timestamp; // unix timestamp in milliseconds
    }
}
impl Order {
    pub fn validate_timestamp(&self) {
        let min_timestamp = chrono::Utc::now().timestamp_millis() - 60000; // 1 minute buffer
        if self.timestamp < min_timestamp.unsigned_abs() {
            panic!("order is too old");
        }
    }
    pub fn validate_signature(&self, signature: Signature, user: Address) {
        let order_hash = self.eip712_signing_hash(&DSTACK_DOMAIN);
        if user != signature.recover_address_from_prehash(&order_hash).unwrap() {
            panic!("invalid signature")
        };
    }
}
sol! {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct CancelOrder {
        uint32 oid;
        uint64 timestamp;// unix timestamp in milliseconds
    }
}
impl CancelOrder {
    pub fn validate_timestamp(&self) {
        let min_timestamp = chrono::Utc::now().timestamp_millis() - 60000; // 1 minute buffer
        if self.timestamp < min_timestamp.unsigned_abs() {
            panic!("order is too old");
        }
    }
    pub fn validate_signature(&self, signature: Signature, user: Address) {
        let order_hash = self.eip712_signing_hash(&DSTACK_DOMAIN);
        if user != signature.recover_address_from_prehash(&order_hash).unwrap() {
            panic!("invalid signature")
        };
    }
}

pub fn new_order(
    warehouse: &mut Warehouse,
    orderbook_manager: &mut OrderBookManager,
    user: Address,
    order: Order,
    signature: Signature,
) -> (Qty, Qty, Option<OrderId>) {
    // validate signature
    order.validate_signature(signature, user);
    // validate timestamp
    order.validate_timestamp();
    // validate inventory state
    let user_inventory = warehouse.inventories.entry(user).or_default();

    // we don't need to validate taker inventory state since they're on margin
    if !user_inventory.is_taker {
        if order.is_bid && Qty(order.qty * order.price).gt(&user_inventory.net_usdc()) {
            panic!("user does not have enough usdc");
        } else if Qty(order.qty).gt(&user_inventory.net_eth()) {
            panic!("user does not have enough eth");
        }
    };
    // execute order
    let (qty_executed, volume_executed, new_order_id, filled_orders, partially_filled_order) =
        match_order(
            orderbook_manager,
            BookId(0),
            order.price,
            Qty(order.qty),
            order.is_bid,
        );
    // update inventory
    if order.is_bid {
        user_inventory
            .eth_balance
            .add_assign(optimized_lob::quantity::Qty(qty_executed.0));
        user_inventory
            .usdc_balance
            .sub_assign(optimized_lob::quantity::Qty(volume_executed.0));
    } else {
        user_inventory.eth_balance.sub_assign(qty_executed);
        user_inventory.usdc_balance.add_assign(volume_executed);
    }

    // update orders
    if let Some(new_order_id) = new_order_id {
        warehouse.add_order(
            new_order_id,
            user,
            Qty(order.qty - qty_executed.0),
            Price::from_u256(order.price, order.is_bid),
        );
    }
    // At this point user is done and we can issue confirmation
    // Update inventories of filled users
    let mut fully_filled_qty = Qty(U256::ZERO);
    if order.is_bid {
        for (order_id, price) in filled_orders.iter() {
            let filled_qty = warehouse.fill_bid(*order_id, *price);
            fully_filled_qty.add_assign(filled_qty);
        }
    } else {
        for (order_id, price) in filled_orders.iter() {
            let filled_qty = warehouse.fill_ask(*order_id, *price);
            fully_filled_qty.add_assign(filled_qty);
        }
    }

    if let Some((order_id, price)) = partially_filled_order {
        // update inventory of partially filled user
        let partially_filled_qty = Qty(qty_executed.0 - fully_filled_qty.0);
        warehouse.partially_fill_order(order_id, partially_filled_qty, price);
    }

    info!(
        "new order: {:?}",
        (user, order.price, order.qty, order.is_bid)
    );
    info!("qty_executed: {}", qty_executed.0);
    info!("volume_executed: {}", volume_executed.0);
    info!("filled_orders: {:?}", filled_orders);
    info!("partially_filled_order: {:?}", partially_filled_order);
    info!("new_order_id: {:?}", new_order_id);
    warehouse.store(); //TODO: do we store here?
    (qty_executed, volume_executed, new_order_id)
}

pub fn cancel_order(
    user: Address,
    cancel: CancelOrder,
    signature: Signature,
    warehouse: &mut Warehouse,
    orderbook_manager: &mut OrderBookManager,
) {
    //validate signature
    cancel.validate_signature(signature, user);
    cancel.validate_timestamp();
    let oid: OrderId = OrderId(cancel.oid);
    let level_id = orderbook_manager.oid_map.get(oid).unwrap().level_id();
    let price = orderbook_manager
        .books
        .get(0)
        .as_deref()
        .unwrap()
        .as_ref()
        .unwrap()
        .level_pool
        .get(level_id)
        .unwrap()
        .price();
    let (_, inventory) = if price.is_bid() {
        warehouse.remove_bid(oid, price)
    } else {
        warehouse.remove_ask(oid)
    };
    if inventory.address != user {
        panic!("order is not owned by user")
    }
    orderbook_manager.remove_order(oid);
    info!("order cancelled: {:?}", (user, oid));
}

pub fn replace_order(
    user: Address,
    order: Order,
    oid: OrderId,
    signature: Signature,
    warehouse: &mut Warehouse,
    orderbook_manager: &mut OrderBookManager,
) -> OrderId {
    // validate signature
    order.validate_signature(signature, user);
    // validate timestamp
    order.validate_timestamp();

    // get new oid
    let new_oid = orderbook_manager.oid_map.next_id();

    let (_, new_inventory) = warehouse.replace_order(
        oid,
        new_oid,
        Qty(order.qty),
        Price::from_u256(order.price, order.is_bid),
    );
    if new_inventory.address != user {
        panic!("order is not owned by user");
    }
    // we don't need to validate taker inventory state since they're on margin
    if !new_inventory.is_taker {
        if new_inventory.eth_liabilities.gt(&new_inventory.eth_balance)
            || new_inventory
                .usdc_liabilities
                .gt(&new_inventory.usdc_balance)
        {
            panic!("user has insufficient balance");
        }
    }
    orderbook_manager.replace_order(oid, new_oid, Qty(order.qty), order.price);
    new_oid
}
