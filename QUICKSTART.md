# Quick Start Guide - Avalanche HTLC Atomic Swap System

This guide will help you deploy and run the atomic swap daemon on Avalanche Fuji testnet.

## Prerequisites

- Rust 1.75+ (2021 edition)
- Node.js 18+ and npm
- Access to Avalanche C-Chain RPC endpoint
- Access to Subnet-EVM RPC endpoint
- Private key with funds on both chains

## Installation

### Option 1: Install from crates.io (Recommended)

```bash
cargo install avalanche-atomic-swap-daemon
```

### Option 2: Build from Source

```bash
git clone https://github.com/yourusername/avalanche-atomic-swap-daemon
cd avalanche-atomic-swap-daemon
cargo build --release
```

The binary will be available at `target/release/avalanche-atomic-swap-daemon`

## Step 1: Get Testnet AVAX

1. Visit the Avalanche Fuji Faucet: https://faucet.avax.network/
2. Request testnet AVAX for your wallet address
3. You'll need AVAX on **both C-Chain and your Subnet** (if testing with a subnet)

## Step 2: Deploy HTLC Contracts

### On C-Chain (Fuji Testnet)

```bash
cd contracts

# Install dependencies
npm install

# Create .env file
cp .env.example .env

# Edit .env and add your private key (without 0x prefix)
# PRIVATE_KEY=your_private_key_here

# Deploy to Fuji C-Chain
npm run deploy:fuji
```

**Save the deployed contract address!** You'll see output like:
```
✅ HTLC deployed to: 0x1234567890abcdef1234567890abcdef12345678
```

### On Echo Testnet (Subnet)

Deploy to the Echo testnet subnet:

```bash
# Deploy to Echo subnet
npm run deploy:echo
```

**Save the deployed contract address!** You'll see output like:
```
✅ HTLC deployed to: 0xabcdef1234567890abcdef1234567890abcdef12
```

## Step 3: Configure the Daemon

Create a `.env` file in the root directory:

```bash
# C-Chain RPC (Fuji testnet)
CCHAIN_RPC=https://api.avax-test.network/ext/bc/C/rpc

# Echo Subnet RPC (testnet)
SUBNET_RPC=https://subnets.avax.network/echo/testnet/rpc

# Daemon private key (can be different from deployment key)
DAEMON_PRIVATE_KEY=0x...

# HTLC contract addresses from Step 2
HTLC_CCHAIN=0x...  # Deployed on Fuji
HTLC_SUBNET=0x...  # Deployed on Echo

# Minimum swap amount (0.01 AVAX in wei)
MIN_AMOUNT_AVAX=10000000000000000

# Polling interval (milliseconds)
POLL_INTERVAL_MS=4000

# Metrics port
METRICS_PORT=8080
```

### Configuration Options

| Parameter | Environment Variable | Default | Description |
|-----------|---------------------|---------|-------------|
| `--cchain-rpc` | `CCHAIN_RPC` | `https://api.avax.network/ext/bc/C/rpc` | C-Chain RPC endpoint |
| `--subnet-rpc` | `SUBNET_RPC` | *Required* | Subnet-EVM RPC endpoint |
| `--daemon-private-key` | `DAEMON_PRIVATE_KEY` | *Required* | Private key for signing transactions |
| `--htlc-cchain` | `HTLC_CCHAIN` | *Required* | C-Chain HTLC contract address |
| `--htlc-subnet` | `HTLC_SUBNET` | *Required* | Subnet HTLC contract address |
| `--min-amount-avax` | `MIN_AMOUNT_AVAX` | `100000000000000000` | Minimum swap amount in wei |
| `--poll-interval-ms` | `POLL_INTERVAL_MS` | `4000` | Block polling interval |
| `--metrics-port` | `METRICS_PORT` | `8080` | Prometheus metrics port |

## Step 4: Fund the Daemon Wallet

The daemon needs AVAX on **both chains** to pay for gas:

1. Send ~1 AVAX to the daemon's address on C-Chain
2. Send ~1 AVAX to the daemon's address on Subnet
3. Get the daemon address: derive it from `DAEMON_PRIVATE_KEY` or check logs on first run

## Step 5: Run the Daemon

### Using Environment Variables (.env file)

```bash
# If installed from crates.io
avalanche-atomic-swap-daemon

# If built from source
./target/release/avalanche-atomic-swap-daemon
```

### Using Command-Line Arguments

```bash
avalanche-atomic-swap-daemon \
  --subnet-rpc "https://subnets.avax.network/echo/testnet/rpc" \
  --daemon-private-key "0x..." \
  --htlc-cchain "0x..." \
  --htlc-subnet "0x..." \
  --min-amount-avax "10000000000000000" \
  --poll-interval-ms 4000 \
  --metrics-port 8080
```

### Using Docker

```dockerfile
FROM rust:1.75 as builder
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

### Logging Levels

Set the log level using the `RUST_LOG` environment variable:

```bash
RUST_LOG=info avalanche-atomic-swap-daemon   # Default
RUST_LOG=debug avalanche-atomic-swap-daemon  # More verbose
RUST_LOG=trace avalanche-atomic-swap-daemon  # Maximum detail
```

## Step 6: Monitor the Daemon

### Check Logs

The daemon will output logs showing:
- Startup and configuration
- Recovered in-flight swaps
- New swap initiations
- Completed swaps

Example output:
```
2024-05-21T14:31:47Z INFO Starting Avalanche Atomic Swap Daemon v0.2.0
2024-05-21T14:31:47Z INFO Recovering in-flight swaps...
2024-05-21T14:31:47Z INFO Recovered 0 in-flight swaps
2024-05-21T14:31:47Z INFO Starting to watch for swaps (C-Chain: block 55610000, Echo: block 26340)
2024-05-21T14:31:47Z INFO Configuration: min_amount=10000000000000000 wei (0.01 AVAX), poll_interval=4000ms
```

### Check Metrics

Visit `http://localhost:8080/metrics` to see Prometheus metrics:
- `avalanche_atomic_swaps_initiated_total` - Total swaps detected
- `avalanche_atomic_swaps_completed_total` - Total swaps completed
- `avalanche_atomic_swaps_in_flight` - Currently active swaps

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'atomic-swap-daemon'
    static_configs:
      - targets: ['localhost:8080']
```

## Step 7: Test a Swap

### Using Test Scripts

The easiest way to test is using the provided scripts:

```bash
cd contracts

# 1. Initiate a test swap on C-Chain
npm run test-swap

# 2. Wait ~15 seconds for daemon to mirror it to Echo

# 3. Check swap status
npm run check-swap

# 4. Claim on Echo
npm run claim-swap

# 5. Daemon automatically claims on C-Chain (watch logs)
```

### Manual Testing

#### C-Chain → Subnet Swap

1. **User locks funds on C-Chain:**
```solidity
// Generate a secret and hashlock
secret = keccak256("my_secret_123")
hashlock = keccak256(secret)

// Lock on C-Chain HTLC
HTLC.lock{value: 0.05 ether}(
    0.05 ether,
    hashlock,
    block.timestamp + 3600  // 1 hour timelock
)
```

2. **Daemon detects and mirrors on Subnet** (automatic, ~15 seconds)

3. **User claims on Subnet:**
```solidity
HTLC.claim(secret)
```

4. **Daemon claims on C-Chain** (automatic, ~8 seconds)

#### Subnet → C-Chain Swap

Same process, but start by locking on Subnet instead.

### Expected Timeline

- **Detection**: < 1 second
- **Finality Wait**: ~15 seconds (3 block confirmations)
- **Mirroring**: ~5 seconds
- **Claim Detection**: < 1 second
- **Claim Completion**: ~8 seconds
- **Total**: < 30 seconds end-to-end

## Troubleshooting

### Daemon not starting
- Check all environment variables are set
- Verify RPC URLs are accessible
- Ensure private key is valid (with 0x prefix)
- Check `.env` file is in the current directory

### Swaps not being detected
- Verify contract addresses are correct
- Check daemon has gas on both chains
- Review logs for errors
- Ensure swap amount meets minimum threshold
- Verify daemon was started BEFORE swap was initiated

### Daemon not mirroring swaps
- Check daemon wallet has sufficient balance on destination chain
- Verify transaction finality (wait for 3 confirmations)
- Check logs for "Waiting for finality" messages
- Ensure minimum amount threshold is met

### Contract deployment fails
- Verify you have testnet AVAX
- Check private key in contracts/.env
- Ensure RPC endpoint is accessible

### Swaps stuck in-flight
- Check transaction finality on both chains
- Verify the daemon has sufficient funds for gas
- Review timelock values to ensure they haven't expired
- Check Prometheus metrics for `in_flight` count

## Network Information

### Fuji Testnet (C-Chain)
- **RPC**: `https://api.avax-test.network/ext/bc/C/rpc`
- **Chain ID**: 43113
- **Explorer**: https://testnet.snowtrace.io/
- **Faucet**: https://faucet.avax.network/

### Echo Testnet (Subnet)
- **RPC**: `https://subnets.avax.network/echo/testnet/rpc`
- **Chain ID**: 173750
- **Explorer**: https://subnets-test.avax.network/echo
- **Note**: Get AVAX from Fuji faucet first, then bridge to Echo

## Production Deployment

### Security Considerations

- **Private Key Management**: Use secure key management (HSM, KMS, etc.)
- **Minimum Amount**: Set appropriate thresholds to prevent spam
- **Monitoring**: Set up alerts for stuck swaps and anomalies
- **Backup**: Run multiple daemon instances for redundancy
- **Rate Limiting**: Monitor RPC usage to avoid rate limits

### Recommended Setup

1. **Use systemd service** for automatic restart
2. **Set up log rotation** to manage disk space
3. **Configure Prometheus alerts** for critical metrics
4. **Use separate wallets** for deployment and daemon operation
5. **Test thoroughly** on testnet before mainnet deployment

### Example systemd Service

```ini
[Unit]
Description=Avalanche Atomic Swap Daemon
After=network.target

[Service]
Type=simple
User=avalanche
WorkingDirectory=/opt/atomic-swap-daemon
EnvironmentFile=/opt/atomic-swap-daemon/.env
ExecStart=/usr/local/bin/avalanche-atomic-swap-daemon
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## Next Steps

- Set up Prometheus/Grafana for monitoring
- Implement automated testing
- Configure alerts for stuck swaps
- Deploy to mainnet (after thorough testing!)
- Set up backup daemon instances

## Support

For issues or questions:
- Check the main [README.md](README.md)
- Review [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- Check daemon logs for error messages
- Review contract deployment guide: `contracts/DEPLOYMENT.md`

## Additional Resources

- [HTLC Contract Interface](contracts/contracts/HTLC.sol)
- [Test Scripts](contracts/scripts/)
- [Troubleshooting Guide](TROUBLESHOOTING.md)
- [Quick Fix Guide](QUICK_FIX.md)
