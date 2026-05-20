import { ethers } from "hardhat";

async function main() {
  console.log("Deploying HTLC contract...");

  const signers = await ethers.getSigners();
  
  if (signers.length === 0) {
    console.error(" ERROR: No accounts found!");
    console.error("Make sure PRIVATE_KEY is set in your .env file");
    console.error("Example: PRIVATE_KEY=your_private_key_without_0x_prefix");
    process.exit(1);
  }

  const deployer = signers[0];
  console.log("Deploying with account:", deployer.address);

  const balance = await ethers.provider.getBalance(deployer.address);
  console.log("Account balance:", ethers.formatEther(balance), "AVAX");

  if (balance === 0n) {
    console.error("⚠️  WARNING: Account has 0 balance!");
    console.error("Get testnet AVAX from: https://faucet.avax.network/");
  }

  const HTLC = await ethers.getContractFactory("HTLC");
  const htlc = await HTLC.deploy();

  await htlc.waitForDeployment();

  const address = await htlc.getAddress();
  console.log(" HTLC deployed to:", address);
  console.log("\nSave this address for your daemon configuration:");
  console.log(`HTLC_CCHAIN=${address}`);
  console.log(`HTLC_SUBNET=${address}`);
  console.log("\nTo verify on Snowtrace:");
  console.log(`npx hardhat verify --network fuji ${address}`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
