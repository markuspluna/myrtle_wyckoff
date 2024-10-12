// EIP-712 implementation
// might need a domain separator for different orderbooks but unsure
use chrono::Utc;
use ethers::{
    types::{Address, Signature, H256, U256},
    utils::{keccak256, to_checksum},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EIP712Domain {
    name: String,
    version: String,
    verifying_contract: Address,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionCallApproval {
    function_name: String,
    params: Vec<String>,
    timestamp: U256,
}

fn hash_eip712_message(domain: &EIP712Domain, message: &FunctionCallApproval) -> H256 {
    let domain_separator = hash_domain(domain);
    let message_hash = hash_approval(message);

    let encoded = ethers::abi::encode(&[
        ethers::abi::Token::FixedBytes(domain_separator.as_bytes().to_vec()),
        ethers::abi::Token::FixedBytes(message_hash.as_bytes().to_vec()),
    ]);

    H256::from_slice(&keccak256(encoded))
}

fn hash_domain(domain: &EIP712Domain) -> H256 {
    let encoded = ethers::abi::encode(&[
        ethers::abi::Token::String("EIP712Domain".to_string()),
        ethers::abi::Token::String(domain.name.clone()),
        ethers::abi::Token::String(domain.version.clone()),
        ethers::abi::Token::Address(domain.verifying_contract),
    ]);

    H256::from_slice(&keccak256(encoded))
}

fn hash_approval(approval: &FunctionCallApproval) -> H256 {
    let encoded = ethers::abi::encode(&[
        ethers::abi::Token::String("FunctionCallApproval".to_string()),
        ethers::abi::Token::String(approval.function_name.clone()),
        ethers::abi::Token::Array(
            approval
                .params
                .iter()
                .map(|p| ethers::abi::Token::String(p.clone()))
                .collect(),
        ),
        ethers::abi::Token::Uint(approval.timestamp),
    ]);

    H256::from_slice(&keccak256(encoded))
}

fn verify_eip712_approval(
    domain: &EIP712Domain,
    approval: &FunctionCallApproval,
    signature: &Signature,
    expected_signer: Address,
    max_age: u64,
) -> bool {
    // Reconstruct the hash from the provided approval data
    let reconstructed_hash = hash_eip712_message(domain, approval);

    // Verify signature
    let recovered = match signature.recover(reconstructed_hash) {
        Ok(address) => address,
        Err(_) => return false,
    };

    if recovered != expected_signer {
        return false;
    }

    // Check timestamp
    let current_time = U256::from(Utc::now().timestamp());
    let time_difference = current_time.saturating_sub(approval.timestamp);

    time_difference <= U256::from(max_age)
}
