// Overview:
// Responsible for validating and creating settlement orders to be posted as part of the state snapshot

use crate::{
    constants::{USDC_ADDRESS, WETH_ADDRESS},
    cowswap::{CowSwapHook, CowSwapOrder},
    verifier::{
        hash_eip712_message, sign_message, verify_eip712_approval, EIP712Domain,
        FunctionCallApproval,
    },
    warehouse::Warehouse,
};
use chrono::Utc;
use ethers::{
    contract::abigen,
    providers::Provider,
    types::{Address, Signature, U256},
};
use std::{str::FromStr, sync::Arc};

// Create settlement orders to be posted as part of the state snapshot
// TODO: this should probably use cowshed https://github.com/cowdao-grants/cow-shed/tree/main instead of transferring funds to the taker
// Note: We're relying heavily on cowswaps social layer to enforce good behavior here
// * pull_settlment_funds() approval signatures are made public as part of the gossiped pre-hook. A malicious taker&solver could abuse this to steal funds.
//   This is probably not scalable in prod, but it's good enough for now, normally a state lock system would allow us to use encrypted published orders to prevent
//   misuse of approval hooks (only the taker would be able to decrypt and send the order for submission)
// * A malicious solver could submit a batch where the swap will revert but not the pre-hook, leading to lost funds
// Note: A malicious taker could submit settlement orders that will never fill but will cause state updates. This is solved with a state lock system.
pub async fn create_settlement_order(
    warehouse: &Warehouse,
    user: String,
    is_bid: bool,
    eth_amount: u64,
    usdc_amount: u64,
    signature: Signature, // signature of the taker
) -> CowSwapOrder {
    let domain = &EIP712Domain {
        name: "MyrtleWyckoff".to_string(),
        version: "1".to_string(),
        verifying_contract: Address::zero(),
    };
    let approval = FunctionCallApproval {
        function_name: "approve".to_string(),
        params: vec![
            user.to_string(),
            is_bid.to_string(),
            eth_amount.to_string(),
            usdc_amount.to_string(),
        ],
        timestamp: U256::from(0),
    };
    if !verify_eip712_approval(
        domain,
        &approval,
        &signature,
        Address::from_str(&user).unwrap(),
        600,
    ) {
        panic!("invalid signature");
    }
    if !warehouse.is_taker(user.to_string()) {
        panic!("user is not a taker");
    }

    let user_balance = warehouse.get_balance(user.to_string());
    if is_bid && usdc_amount > user_balance.1 as u64 {
        panic!("user does not have enough usdc");
    } else if !is_bid && eth_amount > user_balance.0 as u64 {
        panic!("user does not have enough eth");
    }

    let contract_address = "0x...".parse::<Address>().unwrap(); // TODO: replace with contract address

    let provider: Arc<Provider<ethers::providers::Http>> = Arc::new(
        Provider::<ethers::providers::Http>::try_from(
            "https://mainnet.infura.io/v3/YOUR-PROJECT-ID",
        )
        .unwrap(),
    );
    abigen!(
        DepositRegistryContract,
        "./src/dependancies/DepositRegistryABI.json"
    );
    let deposit_registry_contract = DepositRegistryContract::new(contract_address, provider);
    let settlement_nonce: U256 = deposit_registry_contract
        .settlement_nonce()
        .call()
        .await
        .unwrap();
    // Create EIP712Domain
    let domain = EIP712Domain::new(
        "MyrtleWyckoff".to_string(),
        deposit_registry_contract.address(),
    );

    // Create FunctionCallApproval
    let approval = FunctionCallApproval::new(
        "pull_settlement_funds".to_string(),
        vec![
            eth_amount.to_string(),
            usdc_amount.to_string(),
            settlement_nonce.to_string(),
        ],
        U256::from(Utc::now().timestamp()),
    );
    let secret_key = ""; // Placeholder secret key - TODO: get app specific secret from wherever dstack shoves it
    let hook_signature = sign_message(hash_eip712_message(&domain, &approval), secret_key).unwrap();

    // Get calldata
    let pre_hook_calldata = deposit_registry_contract
        .pull_settlement_funds(
            eth_amount,
            usdc_amount,
            hook_signature.to_vec().into(),
            Address::from_str(&user).unwrap(),
        )
        .calldata()
        .unwrap();

    let pre_hook = CowSwapHook::new(
        deposit_registry_contract.address(),
        pre_hook_calldata,
        "100".to_string(), // TODO: figure out what gas cost is
    );
    let app_data = pre_hook.to_app_data();

    let (sell_token, buy_token, sell_amount, buy_amount) = if is_bid {
        (WETH_ADDRESS, USDC_ADDRESS, eth_amount, usdc_amount)
    } else {
        (USDC_ADDRESS, WETH_ADDRESS, usdc_amount, eth_amount)
    };
    CowSwapOrder::new(
        sell_token.to_string(),
        buy_token.to_string(),
        deposit_registry_contract.address().to_string(), // vault will receive the bought tokens
        sell_amount,
        buy_amount,
        (Utc::now().timestamp() + 300) as u64,
        "0".to_string(),
        "buy".to_string(), //this means the order is strict receive, meaning the taker will keep sell surplus - TODO: check this
        false,
        "erc20".to_string(),
        "erc20".to_string(),
        "eip712".to_string(),
        signature.to_string(),
        user,
        app_data.clone(),
        serde_json::to_vec(&app_data).unwrap_or_default(),
    )
}
