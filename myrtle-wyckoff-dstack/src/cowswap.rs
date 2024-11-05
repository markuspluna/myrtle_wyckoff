use alloy::{
    hex::{self, FromHex, ToHexExt},
    primitives::{Address, Bytes},
    signers::{local::PrivateKeySigner, Signature, Signer},
    sol,
    sol_types::{eip712_domain, SolStruct},
};
use chrono::Utc;

use crate::{
    artifacts::IDepositRegistry::Order,
    constants::{USDC_ADDRESS, WETH_ADDRESS},
};

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

sol! {
    struct CowSwapOrderDigest {
        string sell_token;
        string buy_token;
        string receiver;
        string sell_amount;
        string buy_amount;
        uint64 valid_to;
        string fee_amount;
        string kind;
        bool partially_fillable;
        string sell_token_balance;
        string buy_token_balance;
        string from;
        string app_data;
        string app_data_hash;
    }
}
impl CowSwapOrderDigest {
    pub fn from_settlement_order(
        deposit_registry_contract: &String,
        settlement_order: Order,
        app_data: String,
    ) -> CowSwapOrderDigest {
        let (sell_token, buy_token, sell_amount, buy_amount) = if settlement_order.isBid {
            (
                WETH_ADDRESS,
                USDC_ADDRESS,
                settlement_order.ethAmount,
                settlement_order.usdcAmount,
            )
        } else {
            (
                USDC_ADDRESS,
                WETH_ADDRESS,
                settlement_order.usdcAmount,
                settlement_order.ethAmount,
            )
        };
        CowSwapOrderDigest {
            sell_token: sell_token.to_string(),
            buy_token: buy_token.to_string(),
            receiver: deposit_registry_contract.clone(),
            sell_amount: sell_amount.to_string(),
            buy_amount: buy_amount.to_string(),
            valid_to: (Utc::now().timestamp() + 300) as u64,
            fee_amount: "0".to_string(),
            kind: "buy".to_string(),
            partially_fillable: false,
            sell_token_balance: "erc20".to_string(),
            buy_token_balance: "erc20".to_string(),
            from: deposit_registry_contract.clone(),
            app_data: app_data.clone(),
            app_data_hash: format!("0x{}", hex::encode(serde_json::to_vec(&app_data).unwrap())),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CowSwapOrder {
    sell_token: String,
    buy_token: String,
    receiver: String,
    sell_amount: String,
    buy_amount: String,
    valid_to: u64,
    fee_amount: String,
    kind: String,
    partially_fillable: bool,
    sell_token_balance: String,
    buy_token_balance: String,
    signing_scheme: u8,
    signature: String, // needs to be a signature of the order type digest
    from: String,
    app_data: String,
    app_data_hash: String,
}
impl CowSwapOrder {
    pub async fn from_cowswap_order_digest(
        secret_key: &String,
        cowswap_order_digest: CowSwapOrderDigest,
    ) -> CowSwapOrder {
        // this should likely be declared in a constants file but I'll leave it here since it needs to be computed on-the-fly for a multi-chain setup
        let cowswap_domain = eip712_domain! {
            name: "Gnosis Protocol",
            version: "v2",
            chain_id: 1,
            verifying_contract: Address::from_hex("0x9008D19f58AAbD9eD0D60971565AA8510560ab41".encode_hex()).unwrap(),
        };
        let signer = PrivateKeySigner::from_slice(secret_key.as_bytes()).unwrap();
        let hash = cowswap_order_digest.eip712_signing_hash(&cowswap_domain);

        let signature: Signature = signer.sign_hash(&hash).await.unwrap();
        CowSwapOrder {
            sell_token: cowswap_order_digest.sell_token,
            buy_token: cowswap_order_digest.buy_token,
            receiver: cowswap_order_digest.receiver,
            sell_amount: cowswap_order_digest.sell_amount,
            buy_amount: cowswap_order_digest.buy_amount,
            valid_to: cowswap_order_digest.valid_to,
            fee_amount: cowswap_order_digest.fee_amount,
            kind: cowswap_order_digest.kind,
            partially_fillable: cowswap_order_digest.partially_fillable,
            sell_token_balance: cowswap_order_digest.sell_token_balance,
            buy_token_balance: cowswap_order_digest.buy_token_balance,
            signing_scheme: 3,
            signature: format!("0x{}", hex::encode(signature.as_bytes())),
            from: cowswap_order_digest.from,
            app_data: cowswap_order_digest.app_data,
            app_data_hash: cowswap_order_digest.app_data_hash,
        }
    }
}
