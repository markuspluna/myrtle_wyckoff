// Overview:
// Responsible for creating a state checkpoint to be posted to suave.
// This should be run every 5 seconds with the timer reset every new suave block.
// * encrypts inventory state with dstack shared secret app key
// * grabs settlement orders to be posted
// * grabs current settlement nonce
// * creates a signature of the above data
// * posts the encrypted inventory state, and settlement orders to suave via the Checkpointer contracts checkpoint() function

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::RootProvider,
    rpc::types::TransactionReceipt,
    signers::Signer,
    sol_types::SolStruct,
    transports::http::{Client, Http},
};

use std::sync::Arc;

use crate::{artifacts::ICheckpointer, domains::TOLIMAN_DOMAIN, warehouse::Warehouse};

pub async fn snapshot(
    warehouse: &Warehouse,
    provider: &Arc<
        alloy::providers::fillers::FillProvider<
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::JoinFill<
                    alloy::providers::Identity,
                    alloy::providers::fillers::JoinFill<
                        alloy::providers::fillers::GasFiller,
                        alloy::providers::fillers::JoinFill<
                            alloy::providers::fillers::BlobGasFiller,
                            alloy::providers::fillers::JoinFill<
                                alloy::providers::fillers::NonceFiller,
                                alloy::providers::fillers::ChainIdFiller,
                            >,
                        >,
                    >,
                >,
                alloy::providers::fillers::WalletFiller<EthereumWallet>,
            >,
            RootProvider<Http<Client>>,
            Http<Client>,
            Ethereum,
        >,
    >,
) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
    let checkpointer_contract = ICheckpointer::new(warehouse.checkpoint_contract, provider);
    let checkpoint_nonce = checkpointer_contract
        .inventory_checkpoint_nonce()
        .call()
        .await?
        ._0;

    let settlement_orders_json: Vec<String> = warehouse
        .settlement_orders
        .iter()
        .map(|order| serde_json::to_string(order))
        .collect::<Result<_, _>>()?;

    // Create and sign checkpoint
    let checkpoint = ICheckpointer::Checkpoint {
        nonce: checkpoint_nonce,
        inventory_state: warehouse.get_encrypted_inventory()?,
        settlement_orders: settlement_orders_json,
    };
    let hash = checkpoint.eip712_signing_hash(&TOLIMAN_DOMAIN);
    let signature = warehouse.signer.sign_hash(&hash).await?;
    let k256_sig = signature.to_k256()?.to_bytes().to_vec();

    // Execute transaction and return receipt directly
    Ok(checkpointer_contract
        .checkpoint(k256_sig.into(), checkpoint)
        .send()
        .await?
        .get_receipt()
        .await?)
}
