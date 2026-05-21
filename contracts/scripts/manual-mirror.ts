import { ethers } from "hardhat";
import * as dotenv from "dotenv";
import * as fs from "fs";

dotenv.config({ path: "../.env" });

/**
 * Manually mirror a swap from C-Chain to Echo when daemon missed it
 */
async function main() {
    console.log("\n🔧 Manual Swap Mirror Tool\n");
    console.log("=".repeat(60));

    // Read secret and calculate hashlock
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

    // Check C-Chain swap
    console.log("\n" + "=".repeat(60));
    console.log("Step 1: Reading swap from C-Chain");
    console.log("=".repeat(60));
    
    const htlcAddress = process.env.HTLC_CCHAIN || "0x2eC3332598D1256Cdbd3C2360c06E907B26e2C64";
    const htlcAbi = [
        "function swaps(bytes32) view returns (uint256 amount, address sender, uint256 timelock, bool claimed)",
        "function lock(uint256 amount, bytes32 hashlock, uint256 timelock) external payable"
    ];
    
    const fujiProvider = new ethers.JsonRpcProvider("https://api.avax-test.network/ext/bc/C/rpc");
    const fujiHtlc = new ethers.Contract(htlcAddress, htlcAbi, fujiProvider);
    
    const cchainSwap = await fujiHtlc.swaps(hashlock);
    
    if (cchainSwap.amount === 0n) {
        console.error("❌ No swap found on C-Chain with this hashlock!");
        process.exit(1);
    }
    
    console.log("\n✅ Found swap on C-Chain:");
    console.log("  Amount:", ethers.formatEther(cchainSwap.amount), "AVAX");
    console.log("  Sender:", cchainSwap.sender);
    console.log("  Timelock:", new Date(Number(cchainSwap.timelock) * 1000).toLocaleString());
    console.log("  Claimed:", cchainSwap.claimed);

    // Check Echo swap
    console.log("\n" + "=".repeat(60));
    console.log("Step 2: Checking Echo Subnet");
    console.log("=".repeat(60));
    
    const echoProvider = new ethers.JsonRpcProvider("https://subnets.avax.network/echo/testnet/rpc");
    const echoHtlc = new ethers.Contract(htlcAddress, htlcAbi, echoProvider);
    
    const echoSwap = await echoHtlc.swaps(hashlock);
    
    if (echoSwap.amount > 0n) {
        console.log("\n✅ Swap already exists on Echo!");
        console.log("  Amount:", ethers.formatEther(echoSwap.amount), "AVAX");
        console.log("  No need to mirror.");
        process.exit(0);
    }
    
    console.log("\n❌ Swap does NOT exist on Echo. Mirroring now...");

    // Mirror the swap
    console.log("\n" + "=".repeat(60));
    console.log("Step 3: Locking on Echo Subnet");
    console.log("=".repeat(60));
    
    const daemonKey = process.env.DAEMON_PRIVATE_KEY;
    if (!daemonKey) {
        console.error("❌ DAEMON_PRIVATE_KEY not found in .env");
        process.exit(1);
    }
    
    const daemonWallet = new ethers.Wallet(daemonKey, echoProvider);
    console.log("\nUsing daemon wallet:", daemonWallet.address);
    
    const balance = await echoProvider.getBalance(daemonWallet.address);
    console.log("Balance:", ethers.formatEther(balance), "AVAX");
    
    if (balance < cchainSwap.amount) {
        console.error("❌ Insufficient balance on Echo!");
        console.error("Need:", ethers.formatEther(cchainSwap.amount), "AVAX");
        console.error("Have:", ethers.formatEther(balance), "AVAX");
        process.exit(1);
    }
    
    const echoHtlcWithSigner = echoHtlc.connect(daemonWallet);
    
    try {
        console.log("\n📤 Sending lock transaction...");
        const tx = await echoHtlcWithSigner.lock(
            cchainSwap.amount,
            hashlock,
            cchainSwap.timelock,
            { 
                value: cchainSwap.amount,
                gasLimit: 200000
            }
        );
        
        console.log("Transaction hash:", tx.hash);
        console.log("⏳ Waiting for confirmation...");
        
        const receipt = await tx.wait();
        console.log("✅ Transaction confirmed in block:", receipt.blockNumber);
        
        console.log("\n" + "=".repeat(60));
        console.log("🎉 SUCCESS!");
        console.log("=".repeat(60));
        console.log("\nSwap has been mirrored to Echo Subnet.");
        console.log("You can now claim it using:");
        console.log("\n  npm run claim-swap");
        
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
