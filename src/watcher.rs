use crate::{metrics, state::{SwapDirection, SwapState}, traits::*};
use alloy::primitives::TxHash;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct SwapWatcher {
    cchain: Arc<dyn AvalancheChain>,
    subnet: Arc<dyn AvalancheChain>,
    in_flight: Arc<DashMap<[u8; 32], SwapState>>,
    min_amount: u128,
    poll_ms: u64,
}

impl SwapWatcher {
    pub async fn new(
        cchain: Arc<dyn AvalancheChain>,
        subnet: Arc<dyn AvalancheChain>,
        min_amount: u128,
    ) -> Self {
        Self {
            cchain,
            subnet,
            in_flight: Arc::new(DashMap::new()),
            min_amount,
            poll_ms: 4000,
        }
    }

    pub async fn recover_state(&self, blocks_back: u64) -> eyre::Result<()> {
        let latest_c = self.cchain.get_latest_block().await?;
        let latest_s = self.subnet.get_latest_block().await?;
        let from_c = latest_c.saturating_sub(blocks_back);
        let from_s = latest_s.saturating_sub(blocks_back);

        // Recover C→S
        for ev in self.cchain.get_swap_initiated_events(from_c, latest_c).await? {
            if ev.amount.to::<u128>() >= self.min_amount {
                self.in_flight.insert(ev.hashlock, SwapState::Initiated {
                    direction: SwapDirection::CToS,
                    amount: ev.amount,
                    timelock: ev.timelock,
                });
            }
        }
        // Recover S→C
        for ev in self.subnet.get_swap_initiated_events(from_s, latest_s).await? {
            if ev.amount.to::<u128>() >= self.min_amount {
                self.in_flight.insert(ev.hashlock, SwapState::Initiated {
                    direction: SwapDirection::SToC,
                    amount: ev.amount,
                    timelock: ev.timelock,
                });
            }
        }
        metrics::set_in_flight(self.in_flight.len());
        tracing::info!(" Recovered {} in-flight swaps (bidirectional)", self.in_flight.len());
        Ok(())
    }

    pub async fn run(&self) {
        let mut last_c = 0u64;
        let mut last_s = 0u64;

        loop {
            let c_block = self.cchain.get_latest_block().await.unwrap_or(0);
            let s_block = self.subnet.get_latest_block().await.unwrap_or(0);

            if c_block > last_c {
                self.process_c_initiated(last_c + 1, c_block).await;
                self.process_claimed_on_subnet(last_c + 1, c_block).await; // for S→C direction
                last_c = c_block;
            }
            if s_block > last_s {
                self.process_s_initiated(last_s + 1, s_block).await;
                self.process_claimed_on_cchain(last_s + 1, s_block).await; // for C→S direction
                last_s = s_block;
            }

            metrics::set_in_flight(self.in_flight.len());
            sleep(Duration::from_millis(self.poll_ms)).await;
        }
    }

    // C→S direction
    async fn process_c_initiated(&self, from: u64, to: u64) {
        let events = match self.cchain.get_swap_initiated_events(from, to).await {
            Ok(e) => e,
            Err(e) => { tracing::error!("C-Chain initiated fetch failed: {}", e); return; }
        };
        for ev in events {
            if ev.amount.to::<u128>() < self.min_amount { continue; }
            if !self.cchain.is_final(ev.tx_hash).await.unwrap_or(false) { continue; }

            metrics::inc_initiated();
            if let Err(e) = self.subnet.lock_swap(ev.amount, ev.hashlock, ev.timelock).await {
                tracing::error!("Subnet lock failed (C→S): {}", e);
                continue;
            }
            self.in_flight.insert(ev.hashlock, SwapState::Initiated {
                direction: SwapDirection::CToS,
                amount: ev.amount,
                timelock: ev.timelock,
            });
            tracing::info!(" C→S: Locked on Subnet for hashlock {}", hex::encode(ev.hashlock));
        }
    }

    // S→C direction
    async fn process_s_initiated(&self, from: u64, to: u64) {
        let events = match self.subnet.get_swap_initiated_events(from, to).await {
            Ok(e) => e,
            Err(e) => { tracing::error!("Subnet initiated fetch failed: {}", e); return; }
        };
        for ev in events {
            if ev.amount.to::<u128>() < self.min_amount { continue; }
            if !self.subnet.is_final(ev.tx_hash).await.unwrap_or(false) { continue; }

            metrics::inc_initiated();
            if let Err(e) = self.cchain.lock_swap(ev.amount, ev.hashlock, ev.timelock).await {
                tracing::error!("C-Chain lock failed (S→C): {}", e);
                continue;
            }
            self.in_flight.insert(ev.hashlock, SwapState::Initiated {
                direction: SwapDirection::SToC,
                amount: ev.amount,
                timelock: ev.timelock,
            });
            tracing::info!(" S→C: Locked on C-Chain for hashlock {}", hex::encode(ev.hashlock));
        }
    }

    // Claim logic for C→S (user claims on Subnet → daemon claims on C)
    async fn process_claimed_on_cchain(&self, from: u64, to: u64) { /* same as previous process_subnet */ 
        let events = match self.subnet.get_swap_claimed_events(from, to).await {
            Ok(e) => e,
            Err(e) => { tracing::error!("Subnet claimed fetch failed: {}", e); return; }
        };
        for ev in events {
            if let Some(state) = self.in_flight.get(&ev.hashlock) {
                if let SwapState::Initiated { direction: SwapDirection::CToS, .. } = *state {
                    if let Err(e) = self.cchain.claim_swap(ev.secret).await {
                        tracing::error!("C-Chain claim failed: {}", e);
                    } else {
                        self.in_flight.remove(&ev.hashlock);
                        metrics::inc_completed();
                        tracing::info!(" C→S SWAP COMPLETE");
                    }
                }
            }
        }
    }

    // Claim logic for S→C (user claims on C-Chain → daemon claims on Subnet)
    async fn process_claimed_on_subnet(&self, from: u64, to: u64) {
        let events = match self.cchain.get_swap_claimed_events(from, to).await {
            Ok(e) => e,
            Err(e) => { tracing::error!("C-Chain claimed fetch failed: {}", e); return; }
        };
        for ev in events {
            if let Some(state) = self.in_flight.get(&ev.hashlock) {
                if let SwapState::Initiated { direction: SwapDirection::SToC, .. } = *state {
                    if let Err(e) = self.subnet.claim_swap(ev.secret).await {
                        tracing::error!("Subnet claim failed: {}", e);
                    } else {
                        self.in_flight.remove(&ev.hashlock);
                        metrics::inc_completed();
                        tracing::info!(" S→C SWAP COMPLETE");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_direction_enum() {
        assert_eq!(std::mem::size_of::<SwapDirection>(), 1);
    }

    #[tokio::test]
    async fn test_metrics_are_registered() {
        crate::metrics::init_metrics();
        crate::metrics::inc_initiated();
    }
}