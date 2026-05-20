# Quick Start Guide - Avalanche HTLC Atomic Swap System

This guide will help you deploy and run the atomic swap daemon on Avalanche Fuji testnet.

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

# Minimum swap amount (0.1 AVAX in wei)
MIN_AMOUNT_AVAX=100000000000000000

# Polling interval (milliseconds)
POLL_INTERVAL_MS=4000

# Metrics port
METRICS_PORT=8080
```

## Step 4: Fund the Daemon Wallet

The daemon needs AVAX on **both chains** to pay for gas:

1. Send ~1 AVAX to the daemon's address on C-Chain
2. Send ~1 AVAX to the daemon's address on Subnet
3. Get the daemon address: derive it from `DAEMON_PRIVATE_KEY` or check logs on first run

## Step 5: Build and Run the Daemon

```bash
# Build the Rust daemon
cargo build --release

# Run the daemon
cargo run --release
```

Or with environment variables inline:

```bash
CCHAIN_RPC=https://api.avax-test.network/ext/bc/C/rpc \
SUBNET_RPC=https://your-subnet-rpc \
DAEMON_PRIVATE_KEY=0x... \
HTLC_CCHAIN=0x... \
HTLC_SUBNET=0x... \
cargo run --release
```

## Step 6: Monitor the Daemon

### Check Logs

The daemon will output logs showing:
- Startup and configuration
- Recovered in-flight swaps
- New swap initiations
- Completed swaps

### Check Metrics

Visit `http://localhost:8080/metrics` to see Prometheus metrics:
- `avalanche_atomic_swaps_initiated_total`
- `avalanche_atomic_swaps_completed_total`
- `avalanche_atomic_swaps_in_flight`

## Step 7: Test a Swap

### C-Chain → Subnet Swap

1. **User locks funds on C-Chain:**
```solidity
// Generate a secret and hashlock
secret = keccak256("my_secret_123")
hashlock = keccak256(secret)

// Lock on C-Chain HTLC
HTLC.lock{value: 0.5 ether}(
    0.5 ether,
    hashlock,
    block.timestamp + 3600  // 1 hour timelock
)
```

2. **Daemon detects and mirrors on Subnet** (automatic)

3. **User claims on Subnet:**
```solidity
HTLC.claim(secret)
```

4. **Daemon claims on C-Chain** (automatic)

### Subnet → C-Chain Swap

Same process, but start by locking on Subnet instead.

## Troubleshooting

### Daemon not starting
- Check all environment variables are set
- Verify RPC URLs are accessible
- Ensure private key is valid (with 0x prefix)

### Swaps not being detected
- Verify contract addresses are correct
- Check daemon has gas on both chains
- Review logs for errors
- Ensure swap amount meets minimum threshold

### Contract deployment fails
- Verify you have testnet AVAX
- Check private key in contracts/.env
- Ensure RPC endpoint is accessible

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

## Next Steps

- Set up Prometheus/Grafana for monitoring
- Implement automated testing
- Configure alerts for stuck swaps
- Deploy to mainnet (after thorough testing!)

## Support

For issues or questions:
- Check the main README.md
- Review contract deployment guide: `contracts/DEPLOYMENT.md`
- Check daemon logs for error messages
