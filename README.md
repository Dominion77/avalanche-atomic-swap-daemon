# avalanche-atomic-swap-daemon v0.1.0

Production HTLC atomic swap daemon between Avalanche **C-Chain** and any **Subnet-EVM**.

**Bidirectional + Prometheus metrics**

### New in v0.2
- Full C-Chain ↔ Subnet-EVM in both directions
- Prometheus `/metrics` endpoint (port 8080 by default)
- GitHub Actions CI
- Basic test suite

**Metrics exposed**:
- `avalanche_atomic_swaps_initiated_total`
- `avalanche_atomic_swaps_completed_total`
- `avalanche_atomic_swaps_in_flight`

Run with `METRICS_PORT=9090` and scrape with Prometheus/Grafana.

**Production ready** for liquidity providers, bridge teams, and institutional subnet operators.

**Why it exists**: Warp + Teleporter give you the pipes. This daemon is the automated counterparty that finishes the swap without a human watching the screen.

## Quick Start

1. Deploy the `HTLC.sol` contract (below) on **both** chains and note the addresses.
2. Fund the daemon private key on **both** chains.
3. Run:

```bash
export CCHAIN_RPC=https://api.avax.network/ext/bc/C/rpc
export SUBNET_RPC=https://your-subnet-rpc
export DAEMON_PRIVATE_KEY=0x...
export HTLC_CCHAIN=0x...
export HTLC_SUBNET=0x...
cargo run