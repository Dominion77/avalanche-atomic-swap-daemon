import { ethers } from "hardhat";
import * as dotenv from "dotenv";
import * as fs from "fs";

// Load environment variables
dotenv.config({ path: "../.env" });

async function main() {
    console.log("\n🎯 Claiming Swap on Echo Subnet\n");
    console.log("=".repeat(60));

    // Get signer
    const [signer] = await ethers.getSigners();
    console.log("Using account:", signer.address);

    // Check balance
    const balance = await ethers.provider.getBalance(signer.address);
    console.log("Balance:", ethers.formatEther(balance), "AVAX");

    // Read secret from file
    let secret: string;
    try {
        secret = fs.readFileSync('claim-secret.txt', 'utf8').trim();
        console.log("\n📖 Secret loaded from claim-secret.txt");
    } catch (error) {
        console.error("\n❌ Could not read claim-secret.txt");
        console.error("Make sure you ran test-swap.ts first!");
        process.exit(1);
    }

    const hashlock = ethers.keccak256(secret);
    console.log("Secret:", secret);
    console.log("Hashlock:", hashlock);

    // Contract setup
    const htlcAddress = process.env.HTLC_SUBNET || "0x2eC3332598D1256Cdbd3C2360c06E907B26e2C64";
    const htlcAbi = [
        "function claim(bytes32 secret) external",
        "event SwapClaimed(bytes32 indexed hashlock, bytes32 secret)"
    ];

    const htlc = new ethers.Contract(htlcAddress, htlcAbi, signer);

    console.log("\n" + "=".repeat(60));
    console.log("Claiming funds on Echo Subnet");
    console.log("=".repeat(60));

    try {
        const tx = await htlc.claim(secret, {
            gasLimit: 150000
        });
        
        console.log("📤 Transaction sent:", tx.hash);
        console.log("⏳ Waiting for confirmation...");
        
        const receipt = await tx.wait();
        console.log("✅ Transaction confirmed in block:", receipt.blockNumber);

        console.log("\n" + "=".repeat(60));
        console.log("STEP 2: Watch your daemon logs!");
        console.log("=".repeat(60));
        console.log("\nThe daemon should:");
        console.log("1. Detect the SwapClaimed event on Echo");
        console.log("2. Extract the secret from the transaction");
        console.log("3. Automatically claim on C-Chain using the same secret");
        console.log("4. Log: '🎉 C→S SWAP COMPLETE'");

        console.log("\n" + "=".repeat(60));
        console.log("📊 Check Results");
        console.log("=".repeat(60));
        console.log("\n1. Daemon logs (should show swap completion)");
        console.log("2. Metrics: http://localhost:8080/metrics");
        console.log("   - avalanche_atomic_swaps_completed_total should increase");
        console.log("3. Echo explorer: https://subnets-test.avax.network/echo/tx/" + tx.hash);
        console.log("4. Check your balance on both chains");

        console.log("\n🎉 Claim successful!");
        console.log("\n✅ The atomic swap should complete automatically!");
        console.log("Watch the daemon logs for the final claim on C-Chain.");

    } catch (error: any) {
        console.error("\n❌ Error:", error.message);
        if (error.message.includes("already claimed") || error.message.includes("amount > 0")) {
            console.error("\n💡 This swap may have already been claimed.");
        }
        if (error.message.includes("timelock")) {
            console.error("\n⏰ The timelock may have expired.");
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
