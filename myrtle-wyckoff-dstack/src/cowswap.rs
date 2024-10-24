use ethers::types::{Address, Bytes};

#[derive(Debug)]
pub struct CowSwapHook {
    target: Address,
    call_data: Bytes,
    gas_limit: String,
}
impl CowSwapHook {
    pub fn new(target: Address, call_data: Bytes, gas_limit: String) -> CowSwapHook {
        CowSwapHook {
            target,
            call_data,
            gas_limit,
        }
    }
    pub fn to_app_data(&self) -> String {
        format!(
            "{{\"version\":\"1.3.0\",\"metadata\":{{\"hooks\":{{\"pre\":[{{\"target\": \"{}\", \"call_data\": \"{}\", \"gas_limit\": \"{}\"}}], \"post\":[]}}}}}}",
            self.target, self.call_data, self.gas_limit
        )
    }
}

pub struct CowSwapOrder {
    sell_token: String,
    buy_token: String,
    receiver: String,
    sell_amount: u64,
    buy_amount: u64,
    valid_to: u64,
    fee_amount: String,
    kind: String,
    partially_fillable: bool,
    sell_token_balance: String,
    buy_token_balance: String,
    signing_scheme: String,
    signature: String,
    from: String,
    app_data: String,
    app_data_hash: Vec<u8>,
}
impl CowSwapOrder {
    pub fn new(
        sell_token: String,
        buy_token: String,
        receiver: String,
        sell_amount: u64,
        buy_amount: u64,
        valid_to: u64,
        fee_amount: String,
        kind: String,
        partially_fillable: bool,
        sell_token_balance: String,
        buy_token_balance: String,
        signing_scheme: String,
        signature: String,
        from: String,
        app_data: String,
        app_data_hash: Vec<u8>,
    ) -> CowSwapOrder {
        CowSwapOrder {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            fee_amount,
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
            signing_scheme,
            signature,
            from,
            app_data,
            app_data_hash,
        }
    }
}
