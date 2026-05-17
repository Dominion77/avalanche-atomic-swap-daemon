use alloy::primitives::{Address, TxHash, U256};
use async_trait::async_trait;
use eyre::Result;

#[async_trait]
pub trait AvalancheChain: Send + Sync + 'static {
    async fn get_latest_block(&self) -> Result<u64>;
    async fn lock_swap(&self, amount: U256, hashlock: [u8; 32], timelock: u64) -> Result<TxHash>;
    async fn claim_swap(&self, secret: [u8; 32]) -> Result<TxHash>;
    async fn get_swap_initiated_events(&self, from: u64, to: u64) -> Result<Vec<SwapInitiatedEvent>>;
    async fn get_swap_claimed_events(&self, from: u64, to: u64) -> Result<Vec<SwapClaimedEvent>>;
    async fn is_final(&self, tx_hash: TxHash) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct SwapInitiatedEvent {
    pub hashlock: [u8; 32],
    pub amount: U256,
    pub sender: Address,
    pub timelock: u64,
    pub tx_hash: TxHash,
}

#[derive(Debug, Clone)]
pub struct SwapClaimedEvent {
    pub hashlock: [u8; 32],
    pub secret: [u8; 32],
    pub tx_hash: TxHash,
}