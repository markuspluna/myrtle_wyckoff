use alloy::{primitives::Address, sol, sol_types::eip712_domain};
use chrono::Utc;
use serde::{Deserialize, Serialize};

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
