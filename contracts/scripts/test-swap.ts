import { ethers } from "hardhat";
import * as dotenv from "dotenv";

// Load environment variables
dotenv.config({ path: "../.env" });

async function main() {
    console.log("\n🧪 Testing Atomic Swap on Fuji C-Chain → Echo Subnet\n");
    console.log("=".repeat(60));

    // Get signer
    const [signer] = await ethers.getSigners();
    console.log("Using account:", signer.address);

    // Check balance
    const balance = await ethers.provider.getBalance(signer.address);
    console.log("Balance:", ethers.formatEther(balance), "AVAX");

    if (balance < ethers.parseEther("0.2")) {
        console.error("\n❌ Insufficient balance! Need at least 0.2 AVAX for testing.");
        console.error("Get testnet AVAX from: https://faucet.avax.network/");
        process.exit(1);
    }

    // Contract setup
    const htlcAddress = process.env.HTLC_CCHAIN || "0x2eC3332598D1256Cdbd3C2360c06E907B26e2C64";
    const htlcAbi = [
        "function lock(uint256 amount, bytes32 hashlock, uint256 timelock) external payable",
        "function claim(bytes32 secret) external",
        "event SwapInitiated(bytes32 indexed hashlock, uint256 amount, address sender, uint256 timelock)",
        "event SwapClaimed(bytes32 indexed hashlock, bytes32 secret)"
    ];

    const htlc = new ethers.Contract(htlcAddress, htlcAbi, signer);

    // Generate secret and hashlock
    const secretText = "my_test_secret_" + Date.now();
    const secret = ethers.id(secretText);
    const hashlock = ethers.keccak256(secret);

    console.log("\n📝 Swap Details:");
    console.log("Secret text:", secretText);
    console.log("Secret (bytes32):", secret);
    console.log("Hashlock:", hashlock);

    // Swap parameters
    const amount = ethers.parseEther("0.05"); // 0.05 AVAX
    const timelock = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now

    console.log("\n💰 Amount:", ethers.formatEther(amount), "AVAX");
    console.log("⏰ Timelock:", new Date(timelock * 1000).toLocaleString());

    console.log("\n" + "=".repeat(60));
    console.log("STEP 1: Locking funds on Fuji C-Chain");
    console.log("=".repeat(60));

    try {
        const tx = await htlc.lock(amount, hashlock, timelock, { 
            value: amount,
            gasLimit: 200000
        });
        
        console.log("📤 Transaction sent:", tx.hash);
        console.log("⏳ Waiting for confirmation...");
        
        const receipt = await tx.wait();
        console.log("✅ Transaction confirmed in block:", receipt.blockNumber);

        console.log("\n" + "=".repeat(60));
        console.log("STEP 2: Watch your daemon logs!");
        console.log("=".repeat(60));
        console.log("\nThe daemon should:");
        console.log("1. Detect the SwapInitiated event on C-Chain");
        console.log("2. Wait for transaction finality");
        console.log("3. Automatically lock the same amount on Echo Subnet");
        console.log("4. Log: '✅ C→S: Locked on Subnet for hashlock...'");

        console.log("\n" + "=".repeat(60));
        console.log("STEP 3: Claim on Echo Subnet");
        console.log("=".repeat(60));
        console.log("\nTo complete the swap, claim on Echo using:");
        console.log("\nSecret:", secret);
        console.log("\nOr run:");
        console.log(`npx hardhat run scripts/claim-swap.ts --network echo`);

        console.log("\n" + "=".repeat(60));
        console.log("📊 Monitor Progress");
        console.log("=".repeat(60));
        console.log("\n1. Daemon logs (watch for swap events)");
        console.log("2. Metrics: http://localhost:8080/metrics");
        console.log("3. C-Chain explorer: https://testnet.snowtrace.io/tx/" + tx.hash);
        console.log("4. Echo explorer: https://subnets-test.avax.network/echo");

        // Save secret for claiming
        console.log("\n💾 Saving secret to claim-secret.txt...");
        const fs = require('fs');
        fs.writeFileSync('claim-secret.txt', secret);
        console.log("✅ Secret saved!");

        console.log("\n🎉 Swap initiated successfully!");
        console.log("\nHashlock:", hashlock);
        console.log("Amount:", ethers.formatEther(amount), "AVAX");
        console.log("\n⏰ You have 1 hour to claim on Echo before timelock expires.");

    } catch (error: any) {
        console.error("\n❌ Error:", error.message);
        if (error.data) {
            console.error("Error data:", error.data);
        }
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
