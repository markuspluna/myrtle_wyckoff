// Overview: Persistent storage for the app state. Gets shoved into the encrypted volume.
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

use alloy::primitives::{Address, Uint};
use core::ops::{AddAssign as AddAssignTrait, SubAssign as SubAssignTrait};
use optimized_lob::{
    order::OrderId, orderbook_manager::OrderBookManager, price::Price, quantity::Qty,
};
use rocket::request;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::{cowswap::CowSwapOrder, orderhere::Order};

const INVENTORY_STORAGE_PATH: &str = "/mnt/encrypted_data/inventories.json";
const DEPOSIT_CONTRACT_STORAGE_PATH: &str = "/mnt/encrypted_data/deposit_contract.json";
const CHECKPOINT_CONTRACT_STORAGE_PATH: &str = "/mnt/encrypted_data/checkpoint_contract.json";
const RPC_API_KEY_STORAGE_PATH: &str = "/mnt/host_data/rpc_api_key.json";

#[derive(Clone, Debug)]
pub struct Inventory {
    pub address: Address,
    pub eth_balance: Qty,
    pub eth_liabilities: Qty,
    pub usdc_balance: Qty,
    pub usdc_liabilities: Qty,
    pub deposit_nonce: u32,
    pub is_taker: bool,
}
impl Inventory {
    pub fn new(
        address: Address,
        eth_balance: Qty,
        eth_liabilities: Qty,
        usdc_balance: Qty,
        usdc_liabilities: Qty,
        deposit_nonce: u32,
        is_taker: bool,
    ) -> Self {
        Inventory {
            address,
            eth_balance,
            eth_liabilities,
            usdc_balance,
            usdc_liabilities,
            deposit_nonce,
            is_taker,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend(&self.address.0);
        buffer.extend(&self.eth_balance.0.to_le_bytes::<32>());
        buffer.extend(&self.eth_liabilities.0.to_le_bytes::<32>());
        buffer.extend(&self.usdc_balance.0.to_le_bytes::<32>());
        buffer.extend(&self.usdc_liabilities.0.to_le_bytes::<32>());
        buffer.extend(self.deposit_nonce.to_le_bytes());
        buffer.extend((self.is_taker as u8).to_le_bytes());
        buffer.resize(153, 0); // Pad with zeros to reach max possible size Address (20 bytes) + eth_balance (32 bytes) + eth_liabilities (32 bytes) + usdc_balance (32 bytes) + usdc_liabilities (32 bytes) + deposit_nonce (4 bytes) + is_taker (1 byte)
        buffer
    }
    pub fn net_eth(&self) -> Qty {
        Qty(self.eth_balance.0 - self.eth_liabilities.0)
    }
    pub fn net_usdc(&self) -> Qty {
        Qty(self.usdc_balance.0.saturating_sub(self.usdc_liabilities.0))
    }
    pub fn to_json(&self) -> String {
        let serializable_inventory = serde_json::json!({
            "address": self.address.to_string(),
            "eth_balance": self.eth_balance.0.to_string(),
            "eth_liabilities": self.eth_liabilities.0.to_string(),
            "usdc_balance": self.usdc_balance.0.to_string(),
            "usdc_liabilities": self.usdc_liabilities.0.to_string(),
            "deposit_nonce": self.deposit_nonce.to_string(),
            "is_taker": self.is_taker.to_string()
        });
        serde_json::to_string(&serializable_inventory).unwrap()
    }
}
impl Default for Inventory {
    fn default() -> Self {
        Inventory::new(
            Address::default(),
            Qty(Uint::ZERO),
            Qty(Uint::ZERO),
            Qty(Uint::ZERO),
            Qty(Uint::ZERO),
            0,
            false,
        )
    }
}

pub struct Warehouse {
    pub inventories: HashMap<Address, Inventory>, // User inventories
    pub oid_qty_by_address: HashMap<Address, HashMap<OrderId, Qty>>, // address, order ids
    pub address_by_oid: HashMap<OrderId, Address>, // order id, address
    pub deposit_contract: String,
    pub checkpoint_contract: String,
    pub rpc_api_key: String,
    pub settlement_orders: Vec<CowSwapOrder>,
    pub shared_secret: String,
}

impl Warehouse {
    pub fn new(shared_secret: String) -> Self {
        Warehouse {
            inventories: HashMap::new(),
            oid_qty_by_address: HashMap::new(),
            address_by_oid: HashMap::new(),
            deposit_contract: String::new(),
            checkpoint_contract: String::new(),
            rpc_api_key: String::new(),
            settlement_orders: Vec::new(),
            shared_secret: String::new(),
        }
    }
    pub async fn load() -> Self {
        let shared_secret: String = reqwest::get("http://dstack-guest/key/<tag>").await?;
        match Self::load_state() {
            Ok(state) => Self {
                inventories: state.inventories,
                deposit_contract: state.deposit_contract,
                checkpoint_contract: state.checkpoint_contract,
                rpc_api_key: state.rpc_api_key,
                oid_qty_by_address: HashMap::new(),
                address_by_oid: HashMap::new(),
                settlement_orders: Vec::new(),
                shared_secret,
            },
            Err(e) => {
                eprintln!("Failed to load state: {}", e);
                Self::new(shared_secret)
            }
        }
    }

    pub fn store(&self) {
        self.save_state().unwrap();
    }

    fn load_state() -> Result<Warehouse, Box<dyn std::error::Error>> {
        // Read the file
        let inventory_file = std::fs::File::open(INVENTORY_STORAGE_PATH)?;

        // Deserialize
        let inventories: HashMap<Address, Inventory> = serde_json::from_reader(inventory_file)?;

        let deposit_contract_file = std::fs::File::open(DEPOSIT_CONTRACT_STORAGE_PATH)?;
        let deposit_contract: String = serde_json::from_reader(deposit_contract_file)?;

        let checkpoint_contract_file = std::fs::File::open(CHECKPOINT_CONTRACT_STORAGE_PATH)?;
        let checkpoint_contract: String = serde_json::from_reader(checkpoint_contract_file)?;

        let rpc_api_key_file = std::fs::File::open(RPC_API_KEY_STORAGE_PATH)?;
        let rpc_api_key: String = serde_json::from_reader(rpc_api_key_file)?;

        Ok(Warehouse {
            inventories,
            deposit_contract,
            checkpoint_contract,
            rpc_api_key,
            ..Default::default()
        })
    }

    //TODO: I don't think we need to encrypt the state we store in the volume since I think it's stored in the TEE, but unsure
    fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(INVENTORY_STORAGE_PATH).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(INVENTORY_STORAGE_PATH)?;
        let serialized_inventories = serde_json::to_string(&self.inventories)?;
        file.write_all(serialized_inventories)?;

        let mut file = std::fs::File::create(DEPOSIT_CONTRACT_STORAGE_PATH)?;
        file.write_all(&self.deposit_contract)?;

        let mut file = std::fs::File::create(CHECKPOINT_CONTRACT_STORAGE_PATH)?;
        file.write_all(&self.checkpoint_contract)?;

        Ok(())
    }

    pub fn add_order(
        &mut self,
        oid: OrderId,
        address: Address,
        qty: Qty,
        price: Price,
    ) -> (Qty, &mut Inventory) {
        let inventory = self.inventories.entry(address).or_default();
        self.oid_qty_by_address
            .entry(inventory.address)
            .or_default()
            .insert(oid, qty);
        self.address_by_oid.insert(oid, inventory.address);

        if price.is_bid() {
            inventory
                .usdc_liabilities
                .add_assign(Qty(qty.0 * price.absolute()));
        } else {
            inventory.eth_liabilities.add_assign(qty);
        };

        (qty, inventory)
    }
    pub fn fill_bid(&mut self, oid: OrderId, price: Price) -> Qty {
        let (qty, inventory) = self.remove_bid(oid, price);
        inventory.eth_balance.add_assign(qty);
        inventory
            .usdc_balance
            .sub_assign(Qty(qty.0 * price.absolute()));
        qty
    }
    //@Dev: this will not validate that the order is owned by a specific user
    pub fn remove_bid(&mut self, oid: OrderId, price: Price) -> (Qty, &mut Inventory) {
        let address = self
            .address_by_oid
            .remove(&oid)
            .expect("Order does not exist");
        let qty = self
            .oid_qty_by_address
            .get_mut(&address)
            .unwrap()
            .remove(&oid)
            .expect("Order is not owned by this user");
        let inventory = self.inventories.entry(address).or_default();
        let usdc_qty = Qty(qty.0 * price.absolute());
        inventory.usdc_liabilities.sub_assign(usdc_qty);
        (qty, inventory)
    }

    pub fn fill_ask(&mut self, oid: OrderId, price: Price) -> Qty {
        let (qty, inventory) = self.remove_ask(oid);
        inventory.eth_balance.sub_assign(qty);
        inventory
            .usdc_balance
            .add_assign(Qty(qty.0 * price.absolute()));
        qty
    }
    //@Dev: this will not validate that the order is owned by a specific user
    pub fn remove_ask(&mut self, oid: OrderId) -> (Qty, &mut Inventory) {
        let address = self
            .address_by_oid
            .remove(&oid)
            .expect("Order does not exist");
        let qty = self
            .oid_qty_by_address
            .get_mut(&address)
            .unwrap()
            .remove(&oid)
            .expect("Order is not owned by this user");
        let inventory = self.inventories.entry(address).or_default();
        inventory.eth_liabilities.sub_assign(qty);
        (qty, inventory)
    }

    pub fn replace_order(
        &mut self,
        oid: OrderId,
        new_oid: OrderId,
        new_qty: Qty,
        price: Price,
    ) -> (Qty, &Inventory) {
        // Get the address first
        let (order_qty, address) = if price.is_bid() {
            let (order_qty, inventory) = self.remove_bid(oid, price);
            (order_qty, inventory.address)
        } else {
            let (order_qty, inventory) = self.remove_ask(oid);
            (order_qty, inventory.address)
        };

        let (_, inventory) = self.add_order(new_oid, address, new_qty, price);
        (order_qty, inventory)
    }

    pub fn partially_fill_order(&mut self, oid: OrderId, qty: Qty, price: Price) {
        let usdc_qty = Qty(qty.0 * price.absolute());
        let (mut remaining_qty, address) = if price.is_bid() {
            let (order_qty, inventory) = self.remove_bid(oid, price);
            inventory.eth_balance.add_assign(qty);
            inventory.usdc_balance.sub_assign(usdc_qty);
            (order_qty, inventory.address)
        } else {
            let (order_qty, inventory) = self.remove_ask(oid);
            inventory.eth_balance.sub_assign(qty);
            inventory.usdc_balance.add_assign(usdc_qty);
            (order_qty, inventory.address)
        };
        remaining_qty.sub_assign(qty);

        self.add_order(oid, address, remaining_qty, price);
    }

    pub fn get_orders(&self, orderbook_manager: &OrderBookManager, user: Address) -> Vec<Order> {
        let user_orders = self.oid_qty_by_address.get(&user).unwrap();
        let level_pool = &orderbook_manager
            .books
            .get(0)
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .level_pool;
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        user_orders
            .iter()
            .map(|(oid, qty)| {
                let order = orderbook_manager.oid_map.get(*oid).unwrap().clone();
                let price = level_pool.get(order.level_id()).unwrap().price();
                Order {
                    price: price.absolute(),
                    qty: qty.0,
                    is_bid: price.is_bid(),
                    timestamp: timestamp,
                }
            })
            .collect()
    }

    pub fn add_settlement_order(&mut self, order: CowSwapOrder) {
        self.settlement_orders.push(order);
    }
    pub fn clear_settlement_orders(&mut self) {
        self.settlement_orders.clear();
    }

    pub fn is_taker(&self, address: Address) -> bool {
        match self.inventories.get(&address) {
            Some(inventory) => inventory.is_taker,
            None => false,
        }
    }
}
