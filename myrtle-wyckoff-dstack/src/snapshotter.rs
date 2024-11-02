// Overview:
// Responsible for creating a state checkpoint to be posted to suave.
// This should be run every 5 seconds with the timer reset every new suave block.
// * encrypts inventory state with dstack shared secret app key
// * grabs settlement orders to be posted
// * grabs current settlement nonce
// * creates a signature of the above data
// * posts the encrypted inventory state, and settlement orders to suave via the Checkpointer contracts checkpoint() function

use aes_gcm::{
    aead::{heapless::Vec as HeaplessVec, AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use alloy::{
    network::Ethereum,
    primitives::Address,
    providers::{fillers, Identity, RootProvider},
    signers::{local::PrivateKeySigner, Signer},
    sol_types::SolStruct,
    transports::http::{Client, Http},
};

use std::sync::Arc;

use crate::{artifacts::ICheckpointer, domains::TOLIMAN_DOMAIN, warehouse::Warehouse};

pub async fn snapshot(
    warehouse: &Warehouse,
    provider: Arc<
        fillers::FillProvider<
            fillers::JoinFill<
                Identity,
                fillers::JoinFill<
                    fillers::GasFiller,
                    fillers::JoinFill<
                        fillers::BlobGasFiller,
                        fillers::JoinFill<fillers::NonceFiller, fillers::ChainIdFiller>,
                    >,
                >,
            >,
            RootProvider<Http<Client>>,
            Http<Client>,
            Ethereum,
        >,
    >,
) {
    // encrypt inventory state
    let shared_secret = "dstack-app-secret"; // TODO: replace with dstack app specific secret

    let key = Key::<Aes256Gcm>::from_slice(shared_secret.as_bytes());

    let cipher = Aes256Gcm::new(&key);
    let encrypted_inventory_state: Vec<u8> =
        warehouse
            .inventories
            .iter()
            .fold(Vec::new(), |mut encrypted_state, inventory| {
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let mut buffer: HeaplessVec<u8, 128> =
                    HeaplessVec::from_slice(&inventory.1.to_bytes()).unwrap();
                cipher
                    .encrypt_in_place(&nonce, b"", &mut buffer)
                    .expect("encryption failure!");
                encrypted_state.extend_from_slice(&buffer);
                encrypted_state
            });

    let contract_address = Address::from_slice(warehouse.checkpoint_contract.as_bytes());
    let checkpointer_contract = ICheckpointer::new(contract_address, provider);
    let checkpoint_nonce = checkpointer_contract
        .inventory_checkpoint_nonce()
        .call()
        .await
        .unwrap()
        ._0;
    let settlement_orders_json: Vec<String> = warehouse
        .settlement_orders
        .iter()
        .map(|order| serde_json::to_string(order).unwrap())
        .collect();
    // create signature
    let checkpoint = ICheckpointer::Checkpoint {
        nonce: checkpoint_nonce,
        inventory_state: encrypted_inventory_state,
        settlement_orders: settlement_orders_json,
    };
    let signer = PrivateKeySigner::from_slice(shared_secret.as_bytes()).unwrap();
    let hash = checkpoint.eip712_signing_hash(&TOLIMAN_DOMAIN);
    let signature = signer.sign_hash(&hash).await.unwrap();
    checkpointer_contract
        .checkpoint(
            signature.to_k256().unwrap().to_bytes().to_vec().into(),
            checkpoint,
        )
        .send()
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();
}
