import { ethers } from "hardhat";
import * as dotenv from "dotenv";
import * as fs from "fs";

dotenv.config({ path: "../.env" });

/**
 * Manually claim on C-Chain (for when daemon misses the Echo claim)
 */
async function main() {
    console.log("\n🎯 Claiming Swap on Fuji C-Chain\n");
    console.log("=".repeat(60));

    // Read secret
    let secret: string;
    let hashlock: string;
    
    try {
        secret = fs.readFileSync('claim-secret.txt', 'utf8').trim();
        hashlock = ethers.keccak256(secret);
        console.log("Secret:", secret);
        console.log("Hashlock:", hashlock);
    } catch (error) {
        console.error("❌ Could not read claim-secret.txt");
        process.exit(1);
    }

    const htlcAddress = process.env.HTLC_CCHAIN || "0x2eC3332598D1256Cdbd3C2360c06E907B26e2C64";
    const htlcAbi = [
        "function swaps(bytes32) view returns (uint256 amount, address sender, uint256 timelock, bool claimed)",
        "function claim(bytes32 secret) external"
    ];

    // Check swap status
    console.log("\n" + "=".repeat(60));
    console.log("Checking C-Chain Swap Status");
    console.log("=".repeat(60));
    
    const provider = new ethers.JsonRpcProvider("https://api.avax-test.network/ext/bc/C/rpc");
    const htlc = new ethers.Contract(htlcAddress, htlcAbi, provider);
    
    const swap = await htlc.swaps(hashlock);
    
    if (swap.amount === 0n) {
        console.error("\n❌ No swap found on C-Chain with this hashlock!");
        process.exit(1);
    }
    
    console.log("\nSwap on C-Chain:");
    console.log("  Amount:", ethers.formatEther(swap.amount), "AVAX");
    console.log("  Sender:", swap.sender);
    console.log("  Timelock:", new Date(Number(swap.timelock) * 1000).toLocaleString());
    console.log("  Claimed:", swap.claimed);
    
    if (swap.claimed) {
        console.log("\n✅ Swap already claimed on C-Chain!");
        console.log("The atomic swap is complete.");
        process.exit(0);
    }

    // Claim the swap
    console.log("\n" + "=".repeat(60));
    console.log("Claiming on C-Chain");
    console.log("=".repeat(60));
    
    const daemonKey = process.env.DAEMON_PRIVATE_KEY;
    if (!daemonKey) {
        console.error("❌ DAEMON_PRIVATE_KEY not found in .env");
        process.exit(1);
    }
    
    const daemonWallet = new ethers.Wallet(daemonKey, provider);
    console.log("\nUsing daemon wallet:", daemonWallet.address);
    
    const balance = await provider.getBalance(daemonWallet.address);
    console.log("Balance:", ethers.formatEther(balance), "AVAX");
    
    const htlcWithSigner = htlc.connect(daemonWallet);
    
    try {
        console.log("\n📤 Sending claim transaction...");
        const tx = await htlcWithSigner.claim(secret, {
            gasLimit: 100000
        });
        
        console.log("Transaction hash:", tx.hash);
        console.log("⏳ Waiting for confirmation...");
        
        const receipt = await tx.wait();
        console.log("✅ Transaction confirmed in block:", receipt.blockNumber);
        
        console.log("\n" + "=".repeat(60));
        console.log("🎉 SUCCESS!");
        console.log("=".repeat(60));
        console.log("\nAtomic swap completed!");
        console.log("Daemon received:", ethers.formatEther(swap.amount), "AVAX on C-Chain");
        console.log("\nC-Chain explorer:", "https://testnet.snowtrace.io/tx/" + tx.hash);
        
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
