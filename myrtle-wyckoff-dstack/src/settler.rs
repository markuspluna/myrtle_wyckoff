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
    abi::Abi,
    contract::Contract,
    providers::{Provider, ProviderExt},
    types::{Address, Signature, U256},
    utils::{keccak256, to_checksum},
};
use ethers::{
    contract::abigen,
    core::{k256::SecretKey, types::H256},
};
use std::{fs, str::FromStr, sync::Arc};

// store settlement orders to be posted as part of the state snapshot
// TODO: this should make use of hooks in order to properly distribute
// the surplus to the taker - probably use cowshed https://github.com/cowdao-grants/cow-shed/tree/main
// TODO: we probably want to gossip these or emit them as events somehow in prod so the api endpoint isn't a dependancy
// NOTE: the hard part of this is the signaturex isn't tied to the order, so it needs to be directly sent to cowswap rather than being gossiped as part of the
// state snapshot - this results in settlement collisions since the "snapshotter" doesn't know about the settlements submitted by other snapshotters
// Final Note: Actually, we will post the order as part of the state snapshot, we're going to rely on cowswaps social layer to detect and punish bad behavior.
// This is probably not scalable in prod, but it's good enough for now, normally a state lock system would allow us to use encrypted published orders to prevent
// Misuse of approval hooks (only the taker would be able to decrypt and send the order for submission)
// Note: we're also relying on cowswap's social layer to ensure that the filler does not submit the hook unless the trade is going to fill
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
        MyrtleWyckoffContract,
        "./src/dependancies/MyrtleWyckoffABI.json"
    );
    let myrtle_wyckoff_contract = MyrtleWyckoffContract::new(contract_address, provider);
    let settlement_nonce: U256 = myrtle_wyckoff_contract
        .get_settlement_nonce()
        .call()
        .await
        .unwrap();
    // Create EIP712Domain
    let domain = EIP712Domain::new(
        "MyrtleWyckoff".to_string(),
        myrtle_wyckoff_contract.address(), // TODO: replace with contract address
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
    let pre_hook_calldata = myrtle_wyckoff_contract
        .pull_settlement_funds(eth_amount, usdc_amount, hook_signature.to_vec().into())
        .calldata()
        .unwrap();

    let pre_hook = CowSwapHook::new(
        myrtle_wyckoff_contract.address(),
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
        myrtle_wyckoff_contract.address().to_string(), // vault will receive the bought tokens
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
