// Overview:
// Responsible for validating and creating settlement orders to be posted as part of the state snapshot

use crate::{
    artifacts::IDepositRegistry,
    cowswap::{CowSwapHook, CowSwapOrder, CowSwapOrderDigest},
    domains::MAINNET_DOMAIN,
    warehouse::Warehouse,
};
use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::Address,
    providers::{fillers, Identity, RootProvider},
    signers::{local::PrivateKeySigner, Signature, Signer},
    sol_types::SolStruct,
    transports::http::{Client, Http},
};

use std::sync::Arc;

// Create settlement orders to be posted as part of the state snapshot
// TODO: this should probably use cowshed https://github.com/cowdao-grants/cow-shed/tree/main instead of transferring funds to the taker
// Note: A malicious taker could submit settlement orders that will never fill but will cause state updates. This is solved with a state lock system.
// Note: There's an annoying order of operations issue here where dstack must sign the pre-hook before the taker can sign the order. This means there's no way to tie approved order emission to inventory snapshot success.
pub async fn create_settlement_order(
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
    user: Address,
    order: IDepositRegistry::Order,
    taker_signature: Signature,
) -> CowSwapOrder {
    // Validate order
    let order_hash = order.eip712_signing_hash(&MAINNET_DOMAIN);
    if user
        != taker_signature
            .recover_address_from_prehash(&order_hash)
            .unwrap()
    {
        panic!("invalid signature")
    };
    let taker_inventory = warehouse
        .inventories
        .get(&user)
        .cloned()
        .unwrap_or_default();
    if !taker_inventory.is_taker {
        panic!("user is not a taker");
    }

    if order.isBid
        && taker_inventory
            .net_usdc()
            .lt(&optimized_lob::quantity::Qty(order.clone().usdcAmount))
    {
        panic!("user does not have enough usdc");
    } else if !order.clone().isBid
        && taker_inventory
            .net_eth()
            .lt(&optimized_lob::quantity::Qty(order.clone().ethAmount))
    {
        panic!("user does not have enough eth");
    }

    let contract_address = "0x...".parse::<Address>().unwrap(); // TODO: replace with contract address

    let deposit_registry_contract = IDepositRegistry::new(contract_address, provider);

    let secret_key = ""; // TODO: get shared app secret key from dstack
    let signer = PrivateKeySigner::from_slice(secret_key.as_bytes()).unwrap();
    let hook_signature = signer.sign_hash(&order_hash).await.unwrap();

    // Get calldata
    let k256_sig = hook_signature.to_k256().unwrap();
    let signature_bytes = k256_sig.to_bytes().to_vec();
    let pre_hook_calldata = deposit_registry_contract
        .pull_settlement_funds(order.clone(), signature_bytes.into())
        .calldata()
        .clone();

    let pre_hook = CowSwapHook::new(
        deposit_registry_contract.address().clone(),
        pre_hook_calldata,
        "100".to_string(), // TODO: figure out what gas cost is
    );
    let app_data = pre_hook.to_app_data();
    CowSwapOrder::from_cowswap_order_digest(CowSwapOrderDigest::from_settlement_order(
        &warehouse.deposit_contract.clone(),
        order,
        app_data,
    ))
    .await
}
