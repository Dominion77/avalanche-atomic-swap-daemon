# Avalanche Atomic Swap Daemon

[![Crates.io](https://img.shields.io/crates/v/avalanche-atomic-swap-daemon.svg)](https://crates.io/crates/avalanche-atomic-swap-daemon)
[![Documentation](https://docs.rs/avalanche-atomic-swap-daemon/badge.svg)](https://docs.rs/avalanche-atomic-swap-daemon/latest/avalanche_atomic_swap_daemon/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A production-ready HTLC (Hash Time-Locked Contract) atomic swap daemon for bidirectional swaps between Avalanche C-Chain and any Subnet-EVM chain. Built with Rust for performance and reliability.

**For liquidity providers, bridge teams, and institutional subnet operators.**

---

## 📦 Installation

```bash
cargo install avalanche-atomic-swap-daemon
```

Or build from source:
```bash
git clone https://github.com/yourusername/avalanche-atomic-swap-daemon
cd avalanche-atomic-swap-daemon
cargo build --release
```

## 🚀 Quick Start

See [QUICKSTART.md](QUICKSTART.md) for complete deployment guide.

```bash
# Configure via environment variables
export SUBNET_RPC="https://subnet.example.com/rpc"
export DAEMON_PRIVATE_KEY="0x..."
export HTLC_CCHAIN="0x..."
export HTLC_SUBNET="0x..."

# Run the daemon
avalanche-atomic-swap-daemon
```

---

## Features

- **Bidirectional Swaps**: Supports both C-Chain → Subnet and Subnet → C-Chain atomic swaps
- **Automatic Recovery**: Recovers in-flight swaps on startup by scanning recent blocks
- **Transaction Finality**: Validates transaction finality before proceeding with swaps
- **Prometheus Metrics**: Built-in metrics endpoint for monitoring swap activity
- **Configurable Thresholds**: Set minimum swap amounts and polling intervals
- **Production Ready**: Async/await architecture with proper error handling and logging

## How It Works

The daemon monitors HTLC contracts on both C-Chain and Subnet-EVM chains:

1. **Initiation**: User locks funds on Chain A with a hashlock
2. **Mirror Lock**: Daemon detects the lock and mirrors it on Chain B
3. **Claim**: User reveals the secret on Chain B to claim funds
4. **Complete**: Daemon uses the revealed secret to claim funds on Chain A

This ensures atomic execution - either both sides complete or neither does.

## Architecture

```
┌─────────────┐         ┌──────────────┐         ┌─────────────┐
│   C-Chain   │ ◄─────► │    Daemon    │ ◄─────► │   Subnet    │
│ HTLC Contract│         │   Watcher    │         │HTLC Contract│
└─────────────┘         └──────────────┘         └─────────────┘
                              │
                              ▼
                        ┌──────────┐
                        │Prometheus│
                        │ Metrics  │
                        └──────────┘
```

### Components

- **SwapWatcher**: Main orchestrator that monitors both chains and coordinates swaps
- **CChainClient**: Handles C-Chain RPC interactions and HTLC operations
- **SubnetClient**: Handles Subnet-EVM RPC interactions and HTLC operations
- **State Management**: Tracks in-flight swaps using DashMap for concurrent access
- **Metrics**: Exposes Prometheus metrics for monitoring

## Quick Start

See [QUICKSTART.md](QUICKSTART.md) for a complete guide to deploying on Avalanche Fuji testnet.

### Prerequisites

- Rust 1.70+ (2021 edition)
- Access to Avalanche C-Chain RPC endpoint
- Access to Subnet-EVM RPC endpoint
- Private key with funds on both chains
- Deployed HTLC contracts on both chains (see [contracts/DEPLOYMENT.md](contracts/DEPLOYMENT.md))

### Build from Source

```bash
git clone <repository-url>
cd avalanche-atomic-swap-daemon
cargo build --release
```

The binary will be available at `target/release/avalanche-atomic-swap-daemon`

## Configuration

Configure the daemon using environment variables or command-line arguments:

### Required Parameters

| Parameter | Environment Variable | Description |
|-----------|---------------------|-------------|
| `--subnet-rpc` | `SUBNET_RPC` | Subnet-EVM RPC endpoint URL |
| `--daemon-private-key` | `DAEMON_PRIVATE_KEY` | Private key for signing transactions |
| `--htlc-cchain` | `HTLC_CCHAIN` | C-Chain HTLC contract address |
| `--htlc-subnet` | `HTLC_SUBNET` | Subnet HTLC contract address |

### Optional Parameters

| Parameter | Environment Variable | Default | Description |
|-----------|---------------------|---------|-------------|
| `--cchain-rpc` | `CCHAIN_RPC` | `https://api.avax.network/ext/bc/C/rpc` | C-Chain RPC endpoint |
| `--min-amount-avax` | `MIN_AMOUNT_AVAX` | `100000000000000000` (0.1 AVAX) | Minimum swap amount in wei |
| `--poll-interval-ms` | `POLL_INTERVAL_MS` | `4000` | Block polling interval in milliseconds |
| `--metrics-port` | `METRICS_PORT` | `8080` | Prometheus metrics server port |

## Usage

### Using Environment Variables

```bash
export SUBNET_RPC="https://subnet.example.com/rpc"
export DAEMON_PRIVATE_KEY="0x..."
export HTLC_CCHAIN="0x..."
export HTLC_SUBNET="0x..."
export MIN_AMOUNT_AVAX="100000000000000000"

./avalanche-atomic-swap-daemon
```

### Using Command-Line Arguments

```bash
./avalanche-atomic-swap-daemon \
  --subnet-rpc "https://subnet.example.com/rpc" \
  --daemon-private-key "0x..." \
  --htlc-cchain "0x..." \
  --htlc-subnet "0x..." \
  --min-amount-avax "100000000000000000" \
  --poll-interval-ms 4000 \
  --metrics-port 8080
```

### Docker (Example)

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/avalanche-atomic-swap-daemon /usr/local/bin/
ENTRYPOINT ["avalanche-atomic-swap-daemon"]
```

```bash
docker build -t atomic-swap-daemon .
docker run -e SUBNET_RPC="..." -e DAEMON_PRIVATE_KEY="..." atomic-swap-daemon
```

## Monitoring

### Prometheus Metrics

The daemon exposes metrics at `http://0.0.0.0:8080/metrics` (configurable port):

- `avalanche_atomic_swaps_initiated_total`: Total number of initiated swaps
- `avalanche_atomic_swaps_completed_total`: Total number of completed swaps
- `avalanche_atomic_swaps_in_flight`: Current number of in-flight swaps

### Example Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'atomic-swap-daemon'
    static_configs:
      - targets: ['localhost:8080']
```

### Logging

The daemon uses structured logging with the `tracing` crate. Set the log level using the `RUST_LOG` environment variable:

```bash
RUST_LOG=info ./avalanche-atomic-swap-daemon
RUST_LOG=debug ./avalanche-atomic-swap-daemon  # More verbose
```

## HTLC Contract Interface

The daemon expects HTLC contracts with the following interface:

```solidity
contract HTLC {
    event SwapInitiated(bytes32 indexed hashlock, uint256 amount, address sender, uint256 timelock);
    event SwapClaimed(bytes32 indexed hashlock, bytes32 secret);
    
    function lock(uint256 amount, bytes32 hashlock, uint256 timelock) external payable;
    function claim(bytes32 secret) external;
}
```

## Security Considerations

- **Private Key Management**: Store private keys securely. Never commit them to version control.
- **Minimum Amount**: Set appropriate minimum swap amounts to prevent spam and ensure economic viability.
- **Transaction Finality**: The daemon validates C-Chain finality using `platform.getTxStatus`. Subnet finality is assumed fast.
- **Timelock Safety**: Ensure sufficient time between locks to allow for network delays and user actions.
- **Monitoring**: Use Prometheus metrics to detect anomalies and ensure the daemon is operating correctly.

## Development

### Running Tests

```bash
cargo test
```

### Code Structure

```
src/
├── main.rs           # Entry point and initialization
├── config.rs         # CLI and environment configuration
├── watcher.rs        # Main swap orchestration logic
├── state.rs          # Swap state definitions
├── traits.rs         # AvalancheChain trait and event types
├── htlc.rs           # Solidity contract bindings
├── metrics.rs        # Prometheus metrics server
└── clients/
    ├── mod.rs        # Client module exports
    ├── cchain.rs     # C-Chain client implementation
    └── subnet.rs     # Subnet-EVM client implementation
```

## Troubleshooting

### Daemon not detecting swaps

- Verify RPC endpoints are accessible
- Check that HTLC contract addresses are correct
- Ensure the daemon's private key has sufficient gas on both chains
- Review logs for connection errors

### Swaps stuck in-flight

- Check transaction finality on both chains
- Verify the daemon has sufficient funds for gas
- Review timelock values to ensure they haven't expired
- Check Prometheus metrics for `in_flight` count

### High gas costs

- Increase `--poll-interval-ms` to reduce polling frequency
- Increase `--min-amount-avax` to filter out small swaps

## License

Dual-licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Author

**Lord Herrschaft**

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## Metadata

- **Version**: 0.2.0
- **Rust Version**: 1.75+
- **License**: MIT OR Apache-2.0
- **Repository**: https://github.com/yourusername/avalanche-atomic-swap-daemon
- **Documentation**: https://docs.rs/avalanche-atomic-swap-daemon
- **Crates.io**: https://crates.io/crates/avalanche-atomic-swap-daemon

### Keywords
`avalanche`, `atomic-swap`, `htlc`, `blockchain`, `defi`

### Categories
- Cryptography (Cryptocurrencies)
- Network Programming

---

**Built with ❤️ for the Avalanche ecosystem**
