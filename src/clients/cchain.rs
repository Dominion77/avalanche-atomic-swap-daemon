use crate::{htlc::*, traits::*};
use alloy::{
    primitives::{Address, FixedBytes, TxHash, U256},
    providers::{Provider, ProviderBuilder},
    network::EthereumWallet,
    rpc::types::Filter,
    signers::local::PrivateKeySigner,
    sol_types::SolEvent,
};
use alloy::sol_types::SolCall;
use async_trait::async_trait;
use eyre::Result;
use reqwest::Client as ReqwestClient;
use serde_json::json;
use url::Url;

pub struct CChainClient {
    provider: alloy::providers::fillers::FillProvider<
        alloy::providers::fillers::JoinFill<
            alloy::providers::Identity,
            alloy::providers::fillers::WalletFiller<EthereumWallet>,
        >,
        alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::Client>>,
        alloy::transports::http::Http<alloy::transports::http::Client>,
        alloy::network::Ethereum,
    >,
    htlc: Address,
    rpc_client: ReqwestClient,
    rpc_url: Url,
}

impl CChainClient {
    pub async fn new(rpc: Url, htlc: Address, signer: PrivateKeySigner) -> Result<Self> {
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .on_http(rpc.clone());
        
        Ok(Self {
            provider,
            htlc,
            rpc_client: ReqwestClient::new(),
            rpc_url: rpc,
        })
    }
}

#[async_trait]
impl AvalancheChain for CChainClient {
    async fn get_latest_block(&self) -> Result<u64> {
        Ok(self.provider.get_block_number().await?)
    }

    async fn lock_swap(&self, amount: U256, hashlock: [u8; 32], timelock: u64) -> Result<TxHash> {
        let call = HTLC::lockCall { 
            amount, 
            hashlock: FixedBytes::from(hashlock), 
            timelock: U256::from(timelock)
        };
        
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(self.htlc)
            .value(amount)
            .input(call.abi_encode().into());
            
        let pending = self.provider.send_transaction(tx).await?;
        let receipt = pending.get_receipt().await?;
        Ok(receipt.transaction_hash)
    }

    async fn claim_swap(&self, secret: [u8; 32]) -> Result<TxHash> {
        let call = HTLC::claimCall { 
            secret: FixedBytes::from(secret)
        };
        
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(self.htlc)
            .input(call.abi_encode().into());
            
        let pending = self.provider.send_transaction(tx).await?;
        let receipt = pending.get_receipt().await?;
        Ok(receipt.transaction_hash)
    }

    async fn get_swap_initiated_events(&self, from: u64, to: u64) -> Result<Vec<SwapInitiatedEvent>> {
        tracing::debug!("🔍 C-Chain: Fetching SwapInitiated events from block {} to {}", from, to);
        tracing::debug!("🔍 C-Chain: Contract address: {}", self.htlc);
        tracing::debug!("🔍 C-Chain: Event signature: {}", hex::encode(SwapInitiated::SIGNATURE));
        
        let filter = Filter::new()
            .address(self.htlc)
            .event(SwapInitiated::SIGNATURE)
            .from_block(from)
            .to_block(to);

        let logs = self.provider.get_logs(&filter).await?;
        tracing::debug!("🔍 C-Chain: Found {} raw logs", logs.len());
        
        let mut events = vec![];

        for (idx, log) in logs.iter().enumerate() {
            tracing::debug!("🔍 C-Chain: Processing log {}: tx={}, block={}", 
                idx + 1,
                log.transaction_hash.map(|h| hex::encode(h)).unwrap_or_else(|| "none".to_string()),
                log.block_number.unwrap_or(0)
            );
            
            if let Ok(decoded) = SwapInitiated::decode_log(&log.inner, false) {
                tracing::debug!("🔍 C-Chain: Successfully decoded event: hashlock={}, amount={}", 
                    hex::encode(decoded.hashlock.0),
                    decoded.amount
                );
                events.push(SwapInitiatedEvent {
                    hashlock: decoded.hashlock.0,
                    amount: decoded.amount,
                    sender: decoded.sender,
                    timelock: decoded.timelock.to::<u64>(),
                    tx_hash: log.transaction_hash.unwrap_or_default(),
                });
            } else {
                tracing::warn!("⚠️  C-Chain: Failed to decode log {}", idx + 1);
            }
        }
        
        tracing::debug!("🔍 C-Chain: Returning {} decoded events", events.len());
        Ok(events)
    }

    async fn get_swap_claimed_events(&self, from: u64, to: u64) -> Result<Vec<SwapClaimedEvent>> {
        let filter = Filter::new()
            .address(self.htlc)
            .event(SwapClaimed::SIGNATURE)
            .from_block(from)
            .to_block(to);

        let logs = self.provider.get_logs(&filter).await?;
        let mut events = vec![];

        for log in logs {
            if let Ok(decoded) = SwapClaimed::decode_log(&log.inner, false) {
                events.push(SwapClaimedEvent {
                    hashlock: decoded.hashlock.0,
                    secret: decoded.secret.0,
                    tx_hash: log.transaction_hash.unwrap_or_default(),
                });
            }
        }
        Ok(events)
    }

    async fn is_final(&self, tx_hash: TxHash) -> Result<bool> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "platform.getTxStatus",
            "params": [tx_hash.to_string()],
            "id": 1
        });

        let resp: serde_json::Value = self
            .rpc_client
            .post(self.rpc_url.clone())
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp["result"]["status"].as_str() == Some("Accepted"))
    }
}
