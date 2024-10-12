// user entry point for interacting with the system
// I'm not entirely sure how this should be set up, because we need to load state from an encrypted volume
// we might want to cache it in memory, at least until the next checkpoint. In this case the program should
// remain running until the next checkpoint is ready, at which point it should save the new state and exit.
// not entirely sure how to implement.

use optimized_lob::orderbook_manager::OrderBookManager;

use crate::warehouse::Warehouse;

pub struct Jtrain {
    pub warehouse: Warehouse,
    pub orderbook_manager: OrderBookManager,
}

impl Jtrain {
    pub fn new() -> Self {
        let warehouse = Warehouse::load();
        let orderbook_manager = OrderBookManager::new();
        Self {
            warehouse,
            orderbook_manager,
        }
    }
    pub fn run(&mut self) {
        loop {
            //TODO: accept orders and stuff ig

            // post snapshot to suave every 5 seconds or so, wait for response, then wait another 5 seconds
        }
    }
}
