// Not actually sure how I wanna set this up yet since unfamiliar with
// encrypted volumes.
// Basically I think we wanna store 2 data structures in the volume.
// 1. A list of users and their balances
// - this needs to be posted to suave POA chain as part of snapshots
// - so some specific data structure might be needed for that?
// - maybe we bitpack it? IDK prob doesn't matter for now
// 2. A list of orders for each user
// - this just needs to be encrypted
// - users need access to their orders
// - we could allow some peaking regarding overall book state
// 3. book state
// 4. Some addresses
//
// Both of these probably need some caching mechanism since we don't
// want to constantly encrypt and decrypt, but again unsure on the volume side

use optimized_lob::{
    order::{OidMap, OrderId},
    orderbook::OrderBook,
};
use std::collections::{HashMap, HashSet};

pub struct Warehouse {
    pub inventories: HashMap<String, (u64, u32, u8)>, // address, balance, deposit nonce, is_taker
    pub orders: HashMap<String, HashSet<OrderId>>,    // address, order ids
    pub books: Vec<Option<OrderBook>>,                // A mapping of book IDs to order books.
    pub oid_map: OidMap,                              // A mapping of order IDs to order objects.
    pub deposit_contract: String,
    pub checkpoint_contract: String,
}

impl Warehouse {
    pub fn new() -> Self {
        Warehouse {
            inventories: HashMap::new(),
            orders: HashMap::new(),
            books: vec![None; 10],
            oid_map: OidMap::new(),
            deposit_contract: String::new(),
            checkpoint_contract: String::new(),
        }
    }
    pub fn load() -> Self {
        // TODO
        Warehouse {
            inventories: HashMap::new(),
            orders: HashMap::new(),
            books: vec![None; 10],
            oid_map: OidMap::new(),
            deposit_contract: String::new(),
            checkpoint_contract: String::new(),
        }
    }

    pub fn store() -> Self {
        // TODO
        Warehouse {
            inventories: HashMap::new(),
            orders: HashMap::new(),
            books: vec![None; 10],
            oid_map: OidMap::new(),
            deposit_contract: String::new(),
            checkpoint_contract: String::new(),
        }
    }

    pub fn add_order(&mut self, address: String, order_id: OrderId) {
        self.orders.entry(address).or_default().insert(order_id);
    }

    pub fn remove_order(&mut self, address: String, order_id: OrderId) {
        self.orders.entry(address).or_default().remove(&order_id);
    }

    pub fn get_orders(&self, address: String) -> HashSet<OrderId> {
        self.orders.get(&address).cloned().unwrap_or_default()
    }

    pub fn get_balance(&self, address: String) -> u64 {
        self.inventories
            .get(&address)
            .cloned()
            .unwrap_or((0, 0, 0))
            .0
    }

    pub fn set_balance(&mut self, address: String, balance: u64) {
        let current = self.inventories.get(&address).cloned().unwrap_or((0, 0, 0));
        self.inventories
            .insert(address, (balance, current.1, current.2));
    }

    pub fn is_taker(&self, address: String) -> bool {
        self.inventories
            .get(&address)
            .cloned()
            .unwrap_or((0, 0, 0))
            .2
            == 1
    }

    pub fn get_deposit_nonce(&self, address: String) -> u32 {
        self.inventories
            .get(&address)
            .cloned()
            .unwrap_or((0, 0, 0))
            .1
    }

    pub fn set_deposit_nonce(&mut self, address: String, nonce: u32) {
        let current = self.inventories.get(&address).cloned().unwrap_or((0, 0, 0));
        self.inventories
            .insert(address, (current.0, nonce, current.2));
    }
}
