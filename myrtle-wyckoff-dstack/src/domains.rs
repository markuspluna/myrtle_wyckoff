// EIP-712 implementation
// might need a domain separator for different orderbooks but unsure

use alloy::{primitives::Address, sol_types::eip712_domain};

pub const MAINNET_DOMAIN: alloy::dyn_abi::Eip712Domain = eip712_domain! {
    name: "MyrtleWyckoff",
    version: "1",
    verifying_contract: Address::ZERO, //TODO replace with deposit contract address
};

pub const TOLIMAN_DOMAIN: alloy::dyn_abi::Eip712Domain = eip712_domain! {
    name: "MyrtleWyckoff",
    version: "1",
    verifying_contract: Address::ZERO, //TODO replace with checkpoint contract address
};

pub const DSTACK_DOMAIN: alloy::dyn_abi::Eip712Domain = eip712_domain! {
    name: "MyrtleWyckoff",
    version: "1",
    verifying_contract: Address::ZERO, //TODO replace with dstack public key
};

// impl FunctionCallApproval {
//     pub fn new(function_name: String, params: Vec<String>, timestamp: U256) -> Self {
//         FunctionCallApproval {
//             function_name,
//             params,
//             timestamp,
//         }
//     }
//     pub fn hash(&self) -> H256 {
//         hash_approval(self)
//     }
// }

// pub fn hash_eip712_message(domain_separator: &H256, message_hash: &H256) -> H256 {
//     let encoded = ethers::abi::encode(&[
//         ethers::abi::Token::FixedBytes(domain_separator.as_bytes().to_vec()),
//         ethers::abi::Token::FixedBytes(message_hash.as_bytes().to_vec()),
//     ]);

//     H256::from_slice(&keccak256(encoded))
// }

// fn hash_domain(domain: &EIP712Domain) -> H256 {
//     let encoded = ethers::abi::encode(&[
//         ethers::abi::Token::String("EIP712Domain".to_string()),
//         ethers::abi::Token::String(domain.name.clone()),
//         ethers::abi::Token::String(domain.version.clone()),
//         ethers::abi::Token::Address(domain.verifying_contract),
//     ]);

//     H256::from_slice(&keccak256(encoded))
// }

// fn hash_approval(approval: &FunctionCallApproval) -> H256 {
//     let encoded = ethers::abi::encode(&[
//         ethers::abi::Token::String("FunctionCallApproval".to_string()),
//         ethers::abi::Token::String(approval.function_name.clone()),
//         ethers::abi::Token::Array(
//             approval
//                 .params
//                 .iter()
//                 .map(|p| ethers::abi::Token::String(p.clone()))
//                 .collect(),
//         ),
//         ethers::abi::Token::Uint(approval.timestamp),
//     ]);

//     H256::from_slice(&keccak256(encoded))
// }

// pub fn hash_encrypted_inventory_state(checkpoint_nonce: U256, inventory_state: &Vec<u8>) -> H256 {
//     let mut encoded = ethers::abi::encode(&[ethers::abi::Token::Uint(checkpoint_nonce)]);

//     encoded.extend_from_slice(inventory_state);

//     H256::from_slice(&keccak256(encoded))
// }

// pub fn verify_eip712_approval(
//     domain: &EIP712Domain,
//     approval: &FunctionCallApproval,
//     signature: &Signature,
//     expected_signer: Address,
//     max_age: u64,
// ) -> bool {
//     // Reconstruct the hash from the provided approval data
//     let reconstructed_hash = hash_eip712_message(&hash_domain(domain), &hash_approval(approval));

//     // Verify signature
//     let recovered = match signature.recover(reconstructed_hash) {
//         Ok(address) => address,
//         Err(_) => return false,
//     };

//     if recovered != expected_signer {
//         return false;
//     }

//     // Check timestamp
//     let current_time = U256::from(Utc::now().timestamp());
//     let time_difference = current_time.saturating_sub(approval.timestamp);

//     time_difference <= U256::from(max_age)
// }

// // Function to sign a message with a secret key
// pub fn sign_message(
//     message: H256,
//     secret_key: &str,
// ) -> Result<Signature, Box<dyn std::error::Error>> {
//     let secret_key = SecretKey::from_slice(&hex::decode(secret_key)?)?;
//     let signing_key = SigningKey::from(secret_key);
//     let (signature, recovery_id) = signing_key.sign_prehash_recoverable(message.as_ref())?;
//     let v = recovery_id.to_byte() as u64 + 27;
//     let r_bytes: [u8; 32] = signature.r().to_bytes().into();
//     let s_bytes: [u8; 32] = signature.s().to_bytes().into();
//     let r = U256::from_big_endian(&r_bytes);
//     let s = U256::from_big_endian(&s_bytes);

//     Ok(Signature { r, s, v })
// }
