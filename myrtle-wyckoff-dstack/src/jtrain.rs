// user entry point for interacting with the system - this is the app state
// I'm not entirely sure how this should be set up, because we need to load state from an encrypted volume
// we might want to cache it in memory, at least until the next checkpoint. In this case the program should
// remain running until the next checkpoint is ready, at which point it should save the new state and exit.
// not entirely sure how to implement.

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{ProviderBuilder, RootProvider},
    rpc::client::ClientBuilder,
    signers::local::PrivateKeySigner,
    transports::http::{reqwest::Url, Client, Http},
};
use optimized_lob::orderbook_manager::OrderBookManager;
use std::sync::Arc;

use crate::warehouse::Warehouse;

pub struct Jtrain {
    pub warehouse: Warehouse,                //TODO: make this thread safe
    pub orderbook_manager: OrderBookManager, //TODO: make this thread safe
    pub provider: Arc<
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
}

impl Jtrain {
    pub async fn new(http: Url) -> Self {
        let warehouse = Warehouse::load().await;
        let orderbook_manager = OrderBookManager::new();
        let client = ClientBuilder::default().http(http); //TODO: revisit this when testing

        let wallet = EthereumWallet::from(warehouse.signer.clone());
        let provider: Arc<
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
        > = Arc::new(
            ProviderBuilder::new()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_client(client),
        );
        Self {
            warehouse,
            orderbook_manager,
            provider,
        }
    }
}
