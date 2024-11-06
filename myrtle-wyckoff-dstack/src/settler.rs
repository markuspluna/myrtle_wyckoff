// Overview:
// Responsible for validating and creating settlement orders to be posted as part of the state snapshot

use crate::{
    artifacts::IDepositRegistry,
    cowswap::{CowSwapHook, CowSwapOrder, CowSwapOrderDigest},
    domains::MAINNET_DOMAIN,
    errors::MwError,
    warehouse::Warehouse,
};
use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::Address,
    providers::RootProvider,
    signers::{Signature, Signer},
    sol_types::SolStruct,
    transports::http::{Client, Http},
};

use std::sync::Arc;

// Create settlement orders to be posted as part of the state snapshot
// TODO: this should probably use cowshed https://github.com/cowdao-grants/cow-shed/tree/main to properly distribute surplus to the taker
// Note: A malicious taker could submit settlement orders that will never fill but will cause state updates. This is solved with a state lock system.
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
) -> Result<CowSwapOrder, MwError> {
    // Validate order
    let order_hash = order.eip712_signing_hash(&MAINNET_DOMAIN);
    let recovered_address = taker_signature
        .recover_address_from_prehash(&order_hash)
        .map_err(|_| MwError::SignatureRecoveryError)?;

    if user != recovered_address {
        return Err(MwError::InvalidSignature);
    }

    let taker_inventory = warehouse
        .inventories
        .get(&user)
        .cloned()
        .unwrap_or_default();
    if !taker_inventory.is_taker {
        return Err(MwError::NotTaker);
    }

    if order.isBid
        && taker_inventory
            .net_usdc()
            .lt(&optimized_lob::quantity::Qty(order.clone().usdcAmount))
    {
        return Err(MwError::InsufficientBalance {
            token: "USDC".to_string(),
        });
    } else if !order.clone().isBid
        && taker_inventory
            .net_eth()
            .lt(&optimized_lob::quantity::Qty(order.clone().ethAmount))
    {
        return Err(MwError::InsufficientBalance {
            token: "ETH".to_string(),
        });
    }

    let deposit_registry_contract = IDepositRegistry::new(warehouse.deposit_contract, provider);

    let hook_signature = warehouse
        .signer
        .sign_hash(&order_hash)
        .await
        .map_err(|_| MwError::SigningError)?;

    let k256_sig = hook_signature
        .to_k256()
        .map_err(|_| MwError::SignatureConversionError)?;

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
    Ok(CowSwapOrder::from_cowswap_order_digest(
        &warehouse.signer,
        CowSwapOrderDigest::from_settlement_order(
            &warehouse.deposit_contract.to_string(),
            order,
            app_data,
        ),
    )
    .await?)
}
