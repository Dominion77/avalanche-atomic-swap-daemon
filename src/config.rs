use alloy::primitives::Address;
use clap::Parser;
use url::Url;

/// Configuration for the atomic swap daemon
#[derive(Parser, Debug)]
#[command(author, version, about = "Avalanche C-Chain ↔ Subnet-EVM Bidirectional HTLC Atomic Swap Daemon")]
pub struct Config {
    /// C-Chain RPC endpoint URL
    #[arg(long, env = "CCHAIN_RPC", default_value = "https://api.avax.network/ext/bc/C/rpc")]
    pub cchain_rpc: Url,

    /// Subnet-EVM RPC endpoint URL
    #[arg(long, env = "SUBNET_RPC")]
    pub subnet_rpc: Url,

    /// Private key for signing transactions (with 0x prefix)
    #[arg(long, env = "DAEMON_PRIVATE_KEY")]
    pub private_key: String,

    /// HTLC contract address on C-Chain
    #[arg(long, env = "HTLC_CCHAIN")]
    pub htlc_cchain: Address,

    /// HTLC contract address on Subnet
    #[arg(long, env = "HTLC_SUBNET")]
    pub htlc_subnet: Address,

    /// Minimum swap amount in wei
    #[arg(long, env = "MIN_AMOUNT_AVAX", default_value = "100000000000000000")]
    pub min_amount: u128,

    /// Block polling interval in milliseconds
    #[arg(long, env = "POLL_INTERVAL_MS", default_value = "4000")]
    pub poll_interval_ms: u64,

    /// Prometheus metrics server port
    #[arg(long, env = "METRICS_PORT", default_value = "8080")]
    pub metrics_port: u16,
}