// Overview:
// Responsible for creating a state checkpoint to be posted to suave.
// This should be run every 5 seconds with the timer reset every new suave block.
// * encrypts inventory state with dstack shared secret app key
// * grabs settlement orders to be posted
// * grabs current settlement nonce
// * creates a signature of the above data
// * posts the encrypted inventory state, and settlement orders to suave via the Checkpointer contracts checkpoint() function

// use aes::Aes256;
// use aes::cipher::{KeyIvInit, StreamCipher};
// use std::collections::HashMap;
// use std::convert::TryInto;

// type Address = String; // Replace with your actual Address type

// fn encrypt_tuple(
//     data: &(i64, i64, u32, u8),
//     key: &[u8; 32],
//     iv: &[u8; 16],
// ) -> Vec<u8> {
//     // Serialize the tuple into a byte array
//     let mut buffer = Vec::new();
//     buffer.extend(&data.0.to_le_bytes());
//     buffer.extend(&data.1.to_le_bytes());
//     buffer.extend(&data.2.to_le_bytes());
//     buffer.push(data.3);

//     // Encrypt the byte array
//     let mut cipher = Aes256::new(key.into(), iv.into());
//     cipher.apply_keystream(&mut buffer);

//     buffer
// }

// fn main() {
//     let mut map: HashMap<Address, (i64, i64, u32, u8)> = HashMap::new();
//     map.insert("address1".to_string(), (123456789, 987654321, 12345, 255));

//     let key = [0u8; 32]; // Replace with your actual key
//     let iv = [0u8; 16];  // Replace with your actual IV

//     for (address, data) in &map {
//         let encrypted_data = encrypt_tuple(data, &key, &iv);
//         println!("Encrypted data for {}: {:?}", address, encrypted_data);
//     }
// }
