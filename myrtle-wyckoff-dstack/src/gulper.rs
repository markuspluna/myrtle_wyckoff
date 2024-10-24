// Overview:
// Responsible for gulping new deposits from the mainnet deposit registry contract

use std::{fs, sync::Arc};

use ethers::{abi::Abi, contract::Contract, providers::Provider, types::Address};

use crate::warehouse::Warehouse;

#[tokio::main]
pub async fn gulp_deposits(warehouse: &mut Warehouse, user: String) -> [u64; 2] {
    // this should use the dstack in-TDX light client so we don't need to trust the RPC
    // but this is a demo/poc so RPC for now
    let provider: Arc<Provider<ethers::providers::Http>> = Arc::new(
        Provider::<ethers::providers::Http>::try_from(
            "https://mainnet.infura.io/v3/YOUR-PROJECT-ID", // project id probably needs to be an env var
        )
        .unwrap(),
    );

    // Contract address and ABI
    let contract_address = Address::from_slice(warehouse.deposit_contract.as_bytes()); // Replace with your contract address
    let file = fs::read_to_string("./dependancies/MyrtleWyckoffABI.json").unwrap();
    let abi: Abi = serde_json::from_value(serde_json::from_str(&file).unwrap()).unwrap();

    // Create contract instance
    let contract = Contract::new(contract_address, abi, provider);

    // Parameters for the get_deposits function
    let nonce = warehouse.get_deposit_nonce(user.clone());

    // Call the get_deposits function
    let deposits: Vec<[u64; 2]> = contract
        .method::<_, Vec<[u64; 2]>>("get_deposits", (nonce, user.clone()))
        .unwrap()
        .call()
        .await
        .unwrap();

    let mut new_deposits: [u64; 2] = [0, 0];
    for deposit in deposits {
        new_deposits[0] += deposit[0];
        new_deposits[1] += deposit[1];
    }

    warehouse.set_deposit_nonce(user.clone(), nonce + 1);
    let user_balance = warehouse.get_balance(user.clone());
    warehouse.set_balance(
        user.clone(),
        user_balance.0 + new_deposits[0] as i64,
        user_balance.1 + new_deposits[1] as i64,
    );
    warehouse.store(); // maybe don't store here?
    new_deposits
}
