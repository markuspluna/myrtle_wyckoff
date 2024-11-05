use alloy::{primitives::Address, signers::Signature, sol, sol_types::SolStruct};

use crate::domains::DSTACK_DOMAIN;

sol! {
    struct Checkpoint {
        uint256 nonce;
        uint8[] inventory_state;
        string[] settlement_orders;
    }
}
sol! {
    struct SettlementOrder {
        uint256 eth_amount;
        uint256 usdc_amount;
        bool is_bid;
        uint256 nonce;
    }
}
sol! {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct UserRequest {
        address user;
        uint64 timestamp;
        string request_type; // "inventory" or "orders"
    }
}
impl UserRequest {
    pub fn validate_timestamp(&self) {
        let min_timestamp = chrono::Utc::now().timestamp_millis() - 60000; // 1 minute buffer
        if self.timestamp < min_timestamp.unsigned_abs() {
            panic!("request is too old");
        }
    }
    pub fn validate_signature(&self, signature: Signature, user: Address) {
        let order_hash = self.eip712_signing_hash(&DSTACK_DOMAIN);
        if user != signature.recover_address_from_prehash(&order_hash).unwrap() {
            panic!("invalid signature")
        };
    }
    pub fn validate_request_type(&self, request_type: &str) {
        if self.request_type != request_type {
            panic!("invalid request type");
        }
    }
}
