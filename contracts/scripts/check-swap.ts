import { ethers } from "hardhat";
import * as dotenv from "dotenv";
import * as fs from "fs";

dotenv.config({ path: "../.env" });

async function main() {
    console.log("\n🔍 Checking Swap Status\n");
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
        "function swaps(bytes32) view returns (uint256 amount, address sender, uint256 timelock, bool claimed)"
    ];

    // Check on Fuji C-Chain
    console.log("\n" + "=".repeat(60));
    console.log("Checking Fuji C-Chain");
    console.log("=".repeat(60));
    
    const fujiProvider = new ethers.JsonRpcProvider("https://api.avax-test.network/ext/bc/C/rpc");
    const fujiHtlc = new ethers.Contract(htlcAddress, htlcAbi, fujiProvider);
    
    try {
        const swap = await fujiHtlc.swaps(hashlock);
        console.log("\nSwap on C-Chain:");
        console.log("  Amount:", ethers.formatEther(swap.amount), "AVAX");
        console.log("  Sender:", swap.sender);
        console.log("  Timelock:", new Date(Number(swap.timelock) * 1000).toLocaleString());
        console.log("  Claimed:", swap.claimed);
        
        if (swap.amount === 0n) {
            console.log("\n❌ No swap found on C-Chain with this hashlock!");
            console.log("The initial lock transaction may have failed or used a different hashlock.");
        } else if (swap.claimed) {
            console.log("\n✅ Swap already claimed on C-Chain");
        } else {
            console.log("\n✅ Swap exists and is unclaimed on C-Chain");
        }
    } catch (error: any) {
        console.error("Error checking C-Chain:", error.message);
    }

    // Check on Echo Subnet
    console.log("\n" + "=".repeat(60));
    console.log("Checking Echo Subnet");
    console.log("=".repeat(60));
    
    const echoProvider = new ethers.JsonRpcProvider("https://subnets.avax.network/echo/testnet/rpc");
    const echoHtlc = new ethers.Contract(htlcAddress, htlcAbi, echoProvider);
    
    try {
        const swap = await echoHtlc.swaps(hashlock);
        console.log("\nSwap on Echo:");
        console.log("  Amount:", ethers.formatEther(swap.amount), "AVAX");
        console.log("  Sender:", swap.sender);
        console.log("  Timelock:", new Date(Number(swap.timelock) * 1000).toLocaleString());
        console.log("  Claimed:", swap.claimed);
        
        if (swap.amount === 0n) {
            console.log("\n❌ No swap found on Echo!");
            console.log("This means the daemon did NOT mirror the lock from C-Chain.");
            console.log("\nPossible reasons:");
            console.log("1. Daemon is not running");
            console.log("2. Daemon doesn't have funds on Echo to lock");
            console.log("3. Daemon didn't detect the C-Chain event");
            console.log("4. Amount is below minimum threshold");
        } else if (swap.claimed) {
            console.log("\n✅ Swap already claimed on Echo");
        } else {
            console.log("\n✅ Swap exists and is unclaimed on Echo");
            console.log("You can claim it now!");
        }
    } catch (error: any) {
        console.error("Error checking Echo:", error.message);
    }

    // Check daemon wallet balance
    console.log("\n" + "=".repeat(60));
    console.log("Checking Daemon Wallet");
    console.log("=".repeat(60));
    
    const daemonKey = process.env.DAEMON_PRIVATE_KEY;
    if (daemonKey) {
        const daemonWallet = new ethers.Wallet(daemonKey);
        console.log("\nDaemon Address:", daemonWallet.address);
        
        try {
            const fujiBalance = await fujiProvider.getBalance(daemonWallet.address);
            console.log("Fuji Balance:", ethers.formatEther(fujiBalance), "AVAX");
            
            if (fujiBalance < ethers.parseEther("0.1")) {
                console.log("⚠️  Low balance on Fuji!");
            }
        } catch (error: any) {
            console.error("Error checking Fuji balance:", error.message);
        }
        
        try {
            const echoBalance = await echoProvider.getBalance(daemonWallet.address);
            console.log("Echo Balance:", ethers.formatEther(echoBalance), "AVAX");
            
            if (echoBalance < ethers.parseEther("0.1")) {
                console.log("⚠️  Low balance on Echo! Daemon needs funds to lock swaps.");
                console.log("Send some AVAX to:", daemonWallet.address);
            }
        } catch (error: any) {
            console.error("Error checking Echo balance:", error.message);
        }
    }

    console.log("\n" + "=".repeat(60));
    console.log("Configuration");
    console.log("=".repeat(60));
    console.log("\nHTLC Address:", htlcAddress);
    console.log("Min Amount:", process.env.MIN_AMOUNT_AVAX, "wei");
    console.log("           ", Number(process.env.MIN_AMOUNT_AVAX || 0) / 1e18, "AVAX");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
