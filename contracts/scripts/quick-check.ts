import { ethers } from "hardhat";
import * as fs from "fs";

async function main() {
    const secret = fs.readFileSync('claim-secret.txt', 'utf8').trim();
    const hashlock = ethers.keccak256(secret);
    
    const htlcAbi = ["function swaps(bytes32) view returns (uint256 amount, address sender, uint256 timelock, bool claimed)"];
    const htlcAddress = "0x2eC3332598D1256Cdbd3C2360c06E907B26e2C64";
    
    const echoProvider = new ethers.JsonRpcProvider("https://subnets.avax.network/echo/testnet/rpc");
    const echoHtlc = new ethers.Contract(htlcAddress, htlcAbi, echoProvider);
    
    const swap = await echoHtlc.swaps(hashlock);
    
    console.log("Hashlock:", hashlock);
    console.log("Echo Amount:", ethers.formatEther(swap.amount), "AVAX");
    
    if (swap.amount > 0n) {
        console.log("✅ Swap EXISTS on Echo - daemon mirrored it!");
    } else {
        console.log("❌ Swap NOT on Echo yet - daemon hasn't mirrored it");
    }
}

main().catch(console.error);
