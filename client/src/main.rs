use core::panic;

use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
// use sqlx::postgres::PgPoolOptions;
use alloy::{
    network::{Ethereum, EthereumSigner},
    providers::{layers::SignerProvider, Provider, ProviderBuilder, ReqwestProvider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::client::RpcClient,
    signers::{
        k256::SecretKey,
        wallet::{LocalWallet, Wallet},
    },
    transports::{http::Http, ipc::IpcConnect, Transport},
};
use eyre::{eyre, Result};
use log;
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    database_url: String,
    rpc_ipc: Option<String>,
    rpc_http_url: Option<String>,
    private_key: Option<String>,
}

enum ProviderSupported {
    Ipc(RootProvider<PubSubFrontend, Ethereum>),
    Http(RootProvider<Http<Client>>),
}

static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv().ok();

    match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("Failed to load config: {}", error),
    }
});

pub fn get_config() -> &'static Config {
    &CONFIG
}

pub async fn get_ipc_provider() -> Result<RootProvider<PubSubFrontend, Ethereum>> {
    let config = get_config();

    if let Some(ipc_path) = &config.rpc_ipc {
        let ipc = IpcConnect::new(ipc_path.clone());
        let ipc_client = RpcClient::connect_pubsub(ipc).await?;

        let provider = RootProvider::<_, Ethereum>::new(ipc_client);

        Ok(provider)
    } else {
        Err(eyre!("IPC path not found"))
    }
}

pub async fn get_http_provider() -> Result<RootProvider<Http<Client>>> {
    let config = get_config();
    if let Some(http_url) = &config.rpc_http_url {
        let rpc_url = http_url.parse()?;

        let rpc_client = RpcClient::new_http(rpc_url);
        let provider = ReqwestProvider::<Ethereum>::new(rpc_client);
        Ok(provider)
    } else {
        Err(eyre!("RPC HTTP URL not found"))
    }
}

async fn get_provider() -> Result<ProviderSupported> {
    if let Ok(provider) = get_ipc_provider().await {
        Ok(ProviderSupported::Ipc(provider))
    } else if let Ok(provider) = get_http_provider().await {
        Ok(ProviderSupported::Http(provider))
    } else {
        Err(eyre!("No provider found"))
    }
}

fn setup_provider_and_execute_logic<P: Provider>(provider: P, config: &Config) {
    let private_key = config.private_key.as_ref().unwrap();
    let private_key_bytes = private_key.as_bytes().into(); // Assuming this conversion is correct
    let private_key = SecretKey::from_bytes(private_key_bytes).unwrap();
    let wallet: LocalWallet = private_key.into();
    let signer = EthereumSigner::from(wallet);

    let signer_provider = ProviderBuilder::new().signer(signer).provider(provider);

    logic(signer_provider);
}

fn logic(provider: impl Provider) {}

#[tokio::main]
async fn main() {
    let config = get_config();

    let provider = get_provider().await.unwrap();

    match provider {
        ProviderSupported::Ipc(provider) => {
            log::info!("Using IPC provider");
            setup_provider_and_execute_logic(provider.boxed(), config)
        }
        ProviderSupported::Http(provider) => {
            log::info!("Using http provider");
            setup_provider_and_execute_logic(provider.boxed(), config)
        }
    }
}
