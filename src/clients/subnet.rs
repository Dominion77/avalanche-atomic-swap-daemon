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
use url::Url;

pub struct SubnetClient {
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
}

impl SubnetClient {
    pub async fn new(rpc: Url, htlc: Address, signer: PrivateKeySigner) -> Result<Self> {
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .on_http(rpc);
            
        Ok(Self { 
            provider, 
            htlc 
        })
    }
}

#[async_trait]
impl AvalancheChain for SubnetClient {
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
                    timelock: decoded.timelock.to::<u64>(),
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

    async fn is_final(&self, _tx_hash: TxHash) -> Result<bool> {
        Ok(true) // Subnet-EVM finality is fast
    }
}
