use crate::{metrics, state::{SwapDirection, SwapState}, traits::*};
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

        // Recover C→S with chunking
        let c_events = self.fetch_initiated_chunked(&*self.cchain, from_c, latest_c).await?;
        for ev in c_events {
            if ev.amount.to::<u128>() >= self.min_amount {
                self.in_flight.insert(ev.hashlock, SwapState::Initiated {
                    direction: SwapDirection::CToS,
                    amount: ev.amount,
                    timelock: ev.timelock,
                });
            }
        }
        
        // Recover S→C with chunking
        let s_events = self.fetch_initiated_chunked(&*self.subnet, from_s, latest_s).await?;
        for ev in s_events {
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

    async fn fetch_initiated_chunked(
        &self,
        chain: &dyn AvalancheChain,
        from: u64,
        to: u64,
    ) -> eyre::Result<Vec<SwapInitiatedEvent>> {
        const CHUNK_SIZE: u64 = 2000; 
        let mut all_events = Vec::new();
        let mut current = from;

        while current <= to {
            let chunk_end = (current + CHUNK_SIZE).min(to);
            
            match chain.get_swap_initiated_events(current, chunk_end).await {
                Ok(events) => all_events.extend(events),
                Err(e) => tracing::warn!("Failed to fetch events from {} to {}: {}", current, chunk_end, e),
            }
            
            current = chunk_end + 1;
            
            // Small delay to avoid rate limiting
            if current <= to {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(all_events)
    }

    pub async fn run(&self) {
        let mut last_c = self.cchain.get_latest_block().await.unwrap_or(0);
        let mut last_s = self.subnet.get_latest_block().await.unwrap_or(0);

        tracing::info!("🚀 Starting to watch for swaps (C-Chain: block {}, Echo: block {})", last_c, last_s);
        tracing::info!("📊 Configuration: min_amount={} wei, poll_interval={}ms", self.min_amount, self.poll_ms);

        loop {
            let c_block = self.cchain.get_latest_block().await.unwrap_or(0);
            let s_block = self.subnet.get_latest_block().await.unwrap_or(0);

            if c_block > last_c {
                let from = last_c + 1;
                let to = c_block.min(from + 2000); // Chunk to avoid RPC limits
                
                tracing::debug!("📡 C-Chain: New blocks detected ({} -> {}), checking for events...", from, to);
                self.process_c_initiated(from, to).await;
                self.process_claimed_on_subnet(from, to).await;
                last_c = to;
            } else {
                tracing::trace!("⏸️  C-Chain: No new blocks (current: {})", c_block);
            }
            
            if s_block > last_s {
                let from = last_s + 1;
                let to = s_block.min(from + 2000); // Chunk to avoid RPC limits
                
                tracing::debug!("📡 Echo: New blocks detected ({} -> {}), checking for events...", from, to);
                self.process_s_initiated(from, to).await;
                self.process_claimed_on_cchain(from, to).await;
                last_s = to;
            } else {
                tracing::trace!("⏸️  Echo: No new blocks (current: {})", s_block);
            }

            metrics::set_in_flight(self.in_flight.len());
            sleep(Duration::from_millis(self.poll_ms)).await;
        }
    }

    // C→S direction
    async fn process_c_initiated(&self, from: u64, to: u64) {
        tracing::debug!("🔍 C→S: Checking blocks {} to {} for SwapInitiated events", from, to);
        
        let events = match self.cchain.get_swap_initiated_events(from, to).await {
            Ok(e) => {
                tracing::debug!("🔍 C→S: Found {} SwapInitiated events", e.len());
                e
            },
            Err(e) => { 
                tracing::error!("C-Chain initiated fetch failed: {}", e); 
                return; 
            }
        };
        
        if events.is_empty() {
            tracing::debug!("🔍 C→S: No events found in this range");
            return;
        }
        
        for (idx, ev) in events.iter().enumerate() {
            tracing::info!("🔍 C→S: Event {}: hashlock={}, amount={} wei, timelock={}, tx={}", 
                idx + 1,
                hex::encode(ev.hashlock),
                ev.amount,
                ev.timelock,
                hex::encode(ev.tx_hash)
            );
            
            // Check minimum amount
            let amount_u128 = ev.amount.to::<u128>();
            tracing::debug!("🔍 C→S: Amount check: {} >= {} (min)?", amount_u128, self.min_amount);
            if amount_u128 < self.min_amount {
                tracing::warn!("⚠️  C→S: Skipping - amount {} below minimum {}", amount_u128, self.min_amount);
                continue;
            }
            
            // Check finality
            tracing::debug!("🔍 C→S: Checking finality for tx {}", hex::encode(ev.tx_hash));
            let is_final = self.cchain.is_final(ev.tx_hash).await.unwrap_or(false);
            tracing::debug!("🔍 C→S: Finality check result: {}", is_final);
            if !is_final {
                tracing::warn!("⚠️  C→S: Skipping - transaction not yet final");
                continue;
            }

            tracing::info!("✅ C→S: Valid swap detected! Locking on Subnet...");
            metrics::inc_initiated();
            
            if let Err(e) = self.subnet.lock_swap(ev.amount, ev.hashlock, ev.timelock).await {
                tracing::error!("❌ Subnet lock failed (C→S): {}", e);
                continue;
            }
            
            self.in_flight.insert(ev.hashlock, SwapState::Initiated {
                direction: SwapDirection::CToS,
                amount: ev.amount,
                timelock: ev.timelock,
            });
            tracing::info!("✅ C→S: Locked on Subnet for hashlock {}", hex::encode(ev.hashlock));
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
    async fn process_claimed_on_cchain(&self, from: u64, to: u64) {
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
                        tracing::info!("🎉 C→S SWAP COMPLETE");
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
