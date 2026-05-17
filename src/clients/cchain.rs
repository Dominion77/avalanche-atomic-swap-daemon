use crate::{htlc::*, traits::*};
use alloy::{
    primitives::{Address, Bytes, TxHash, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::Filter,
    signers::local::PrivateKeySigner,
    sol_types::SolEvent,
};
use eyre::Result;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use url::Url;

pub struct CChainClient {
    provider: Arc<dyn Provider>,
    htlc: Address,
    rpc_client: Client,
    rpc_url: Url,
}

impl CChainClient {
    pub async fn new(rpc: Url, htlc: Address, signer: PrivateKeySigner) -> Result<Self> {
        let provider = ProviderBuilder::new()
            .with_signer(signer)
            .on_http(rpc.clone());
        Ok(Self {
            provider: Arc::new(provider),
            htlc,
            rpc_client: Client::new(),
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
        let call = HTLC::lockCall { amount, hashlock, timelock };
        let tx = self.provider.send_transaction(
            alloy::rpc::types::TransactionRequest::new()
                .to(self.htlc)
                .value(amount)
                .input(Bytes::from(call.abi_encode()))
        ).await?;
        Ok(tx.get_receipt().await?.transaction_hash)
    }

    async fn claim_swap(&self, secret: [u8; 32]) -> Result<TxHash> {
        let call = HTLC::claimCall { secret };
        let tx = self.provider.send_transaction(
            alloy::rpc::types::TransactionRequest::new()
                .to(self.htlc)
                .input(Bytes::from(call.abi_encode()))
        ).await?;
        Ok(tx.get_receipt().await?.transaction_hash)
    }

    async fn get_swap_initiated_events(&self, from: u64, to: u64) -> Result<Vec<SwapInitiatedEvent>> {
        let filter = Filter::new()
            .address(self.htlc)
            .event(SwapInitiated::SIGNATURE)
            .from_block(from)
            .to_block(to);

        let logs = self.provider.get_logs(&filter).await?;
        let mut events = vec![];

        for log in logs {
            if let Ok(decoded) = SwapInitiated::decode_log(&log.inner, false) {
                events.push(SwapInitiatedEvent {
                    hashlock: decoded.hashlock.0,
                    amount: decoded.amount,
                    sender: decoded.sender,
                    timelock: decoded.timelock,
                    tx_hash: log.transaction_hash.unwrap_or_default(),
                });
            }
        }
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