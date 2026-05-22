//! # Avalanche Atomic Swap Daemon
//!
//! A production-ready HTLC (Hash Time-Locked Contract) atomic swap daemon for bidirectional
//! swaps between Avalanche C-Chain and any Subnet-EVM chain.
//!
//! ## Features
//!
//! - **Bidirectional Swaps**: Supports both C-Chain → Subnet and Subnet → C-Chain atomic swaps
//! - **Automatic Recovery**: Recovers in-flight swaps on startup by scanning recent blocks
//! - **Transaction Finality**: Validates transaction finality before proceeding with swaps
//! - **Prometheus Metrics**: Built-in metrics endpoint for monitoring swap activity
//! - **Configurable Thresholds**: Set minimum swap amounts and polling intervals
//!
//! ## Quick Start
//!
//! ```bash
//! cargo install avalanche-atomic-swap-daemon
//! avalanche-atomic-swap-daemon --help
//! ```
//!
//! ## Configuration
//!
//! Configure via environment variables or command-line arguments:
//!
//! ```bash
//! export SUBNET_RPC="https://subnet.example.com/rpc"
//! export DAEMON_PRIVATE_KEY="0x..."
//! export HTLC_CCHAIN="0x..."
//! export HTLC_SUBNET="0x..."
//! avalanche-atomic-swap-daemon
//! ```
//!
//! ## Architecture
//!
//! The daemon monitors HTLC contracts on both chains:
//!
//! 1. **Initiation**: User locks funds on Chain A with a hashlock
//! 2. **Mirror Lock**: Daemon detects the lock and mirrors it on Chain B
//! 3. **Claim**: User reveals the secret on Chain B to claim funds
//! 4. **Complete**: Daemon uses the revealed secret to claim funds on Chain A
//!
//! ## Library Usage
//!
//! While primarily a binary application, the library exposes core components
//! for building custom atomic swap solutions:
//!
//! ```rust,no_run
//! use avalanche_atomic_swap_daemon::{
//!     clients::{CChainClient, SubnetClient},
//!     watcher::SwapWatcher,
//!     config::Config,
//! };
//! use std::sync::Arc;
//!
//! # async fn example() -> eyre::Result<()> {
//! // Configuration would typically come from Config::parse()
//! # let config = Config::parse();
//! # let signer = "0x1234".parse().unwrap();
//! # let cchain = Arc::new(CChainClient::new(config.cchain_rpc, config.htlc_cchain, signer.clone()).await?);
//! # let subnet = Arc::new(SubnetClient::new(config.subnet_rpc, config.htlc_subnet, signer).await?);
//!
//! // Create swap watcher
//! let watcher = SwapWatcher::new(cchain, subnet, config.min_amount).await;
//!
//! // Recover in-flight swaps
//! watcher.recover_state(200).await?;
//!
//! // Start monitoring (this runs forever)
//! watcher.run().await;
//! # Ok(())
//! # }
//! ```

#![doc(html_root_url = "https://docs.rs/avalanche-atomic-swap-daemon/0.2.1")]
#![allow(missing_docs)]

/// Client implementations for C-Chain and Subnet-EVM chains
pub mod clients;

/// Configuration parsing and management
pub mod config;

/// HTLC contract bindings generated from Solidity
pub mod htlc;

/// Prometheus metrics server and collectors
pub mod metrics;

/// Swap state definitions and types
pub mod state;

/// Trait definitions for chain interactions
pub mod traits;

/// Main swap watcher and orchestration logic
pub mod watcher;

// Re-export commonly used types
pub use crate::clients::cchain::CChainClient;
pub use crate::clients::subnet::SubnetClient;
pub use config::Config;
pub use state::{SwapDirection, SwapState};
pub use traits::{AvalancheChain, SwapClaimedEvent, SwapInitiatedEvent};
pub use watcher::SwapWatcher;
