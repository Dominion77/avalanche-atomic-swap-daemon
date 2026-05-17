use alloy::primitives::Address;
use clap::Parser;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about = "Avalanche C-Chain ↔ Subnet-EVM Bidirectional HTLC Atomic Swap Daemon")]
pub struct Config {
    #[arg(long, env = "CCHAIN_RPC", default_value = "https://api.avax.network/ext/bc/C/rpc")]
    pub cchain_rpc: Url,

    #[arg(long, env = "SUBNET_RPC")]
    pub subnet_rpc: Url,

    #[arg(long, env = "DAEMON_PRIVATE_KEY")]
    pub private_key: String,

    #[arg(long, env = "HTLC_CCHAIN")]
    pub htlc_cchain: Address,

    #[arg(long, env = "HTLC_SUBNET")]
    pub htlc_subnet: Address,

    #[arg(long, env = "MIN_AMOUNT_AVAX", default_value = "100000000000000000")]
    pub min_amount: u128,

    #[arg(long, env = "POLL_INTERVAL_MS", default_value = "4000")]
    pub poll_interval_ms: u64,

    #[arg(long, env = "METRICS_PORT", default_value = "8080")]
    pub metrics_port: u16,
}