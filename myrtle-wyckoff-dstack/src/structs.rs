use alloy::sol;

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
