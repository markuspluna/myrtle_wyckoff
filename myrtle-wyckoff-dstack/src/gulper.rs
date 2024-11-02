// Overview:
// Responsible for gulping new deposits from the mainnet deposit registry contract

use std::sync::Arc;

use alloy::{
    network::Ethereum,
    primitives::{Address, U256},
    providers::{fillers, Identity, RootProvider},
    transports::http::{Client, Http},
};
use core::ops::AddAssign as AddAssignTrait;
use optimized_lob::quantity::Qty;

use crate::{
    artifacts::IDepositRegistry,
    warehouse::{Inventory, Warehouse},
};

#[tokio::main]
pub async fn gulp_deposits(
    warehouse: &mut Warehouse,
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
    user: Address,
) -> [U256; 2] {
    // this should use the dstack in-TDX light client rather than the provider so we don't need to trust the RPC
    // but this is a demo/poc so RPC for now

    // Contract address and ABI
    let contract_address = Address::from_slice(warehouse.deposit_contract.as_bytes()); // Create contract instance

    let deposit_registry_contract = IDepositRegistry::new(contract_address, &provider); //pretty sure .into() won't work TODO: revisit

    // Get user inventory
    let inventory = warehouse
        .inventories
        .entry(user)
        .or_insert(Inventory::default());

    // Call the get_deposits function
    let deposits: Vec<[U256; 2]> = deposit_registry_contract
        .get_deposits(inventory.deposit_nonce.clone(), user.clone())
        .call()
        .await
        .unwrap()
        ._0;

    let mut new_deposits: [U256; 2] = [U256::ZERO, U256::ZERO];
    for deposit in deposits {
        new_deposits[0] += deposit[0];
        new_deposits[1] += deposit[1];
    }
    inventory.deposit_nonce += 1;

    inventory
        .eth_balance
        .add_assign(Qty(new_deposits[0].clone()));
    inventory
        .usdc_balance
        .add_assign(Qty(new_deposits[1].clone()));

    warehouse.store(); // maybe don't store here?
    new_deposits
}
