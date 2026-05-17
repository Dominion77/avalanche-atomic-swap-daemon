use alloy::signers::local::PrivateKeySigner;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber;

mod config;
mod traits;
mod htlc;
mod clients;
mod watcher;
mod state;
mod metrics;

use config::Config;
use clients::{cchain::CChainClient, subnet::SubnetClient};
use watcher::SwapWatcher;
use metrics::start_metrics_server;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing::Level::INFO)
        .init();

    let config = Config::parse();
    tracing::info!(" Starting Avalanche Atomic Swap Daemon v0.2.0 (bidirectional + metrics)...");

    metrics::init_metrics();

    let signer: PrivateKeySigner = config.private_key.parse()
        .expect("Invalid private key format");

    let cchain = Arc::new(
        CChainClient::new(config.cchain_rpc.clone(), config.htlc_cchain, signer.clone()).await?,
    );

    let subnet = Arc::new(
        SubnetClient::new(config.subnet_rpc.clone(), config.htlc_subnet, signer).await?,
    );

    let watcher = SwapWatcher::new(cchain, subnet, config.min_amount).await;

    tracing::info!(" Recovering in-flight swaps...");
    watcher.recover_state(200).await?;

    // Spawn metrics server
    let metrics_port = config.metrics_port;
    tokio::spawn(async move {
        if let Err(e) = start_metrics_server(metrics_port).await {
            tracing::error!("Metrics server failed: {}", e);
        }
    });

    watcher.run().await;

    Ok(())
}