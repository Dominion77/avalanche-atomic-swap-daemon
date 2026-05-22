use avalanche_atomic_swap_daemon::{
    CChainClient,
    SubnetClient,
    Config,
    metrics::start_metrics_server,
    SwapWatcher,
};
use alloy::signers::local::PrivateKeySigner;
use clap::Parser;
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let config = Config::parse();
    tracing::info!(" Starting Avalanche Atomic Swap Daemon v0.2.0 (bidirectional + metrics)...");

    avalanche_atomic_swap_daemon::metrics::init_metrics();

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