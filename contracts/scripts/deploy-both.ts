import { ethers } from "hardhat";
import hre from "hardhat";

async function main() {
  const networkName = hre.network.name;
  
  console.log(`\n${"=".repeat(60)}`);
  console.log(`Deploying to ${networkName.toUpperCase()}...`);
  console.log("=".repeat(60));

  const [deployer] = await ethers.getSigners();
  console.log("Deploying with account:", deployer.address);

  const balance = await ethers.provider.getBalance(deployer.address);
  console.log("Account balance:", ethers.formatEther(balance), "AVAX");

  if (balance === 0n) {
    console.log(`  WARNING: Account has 0 balance on ${networkName}!`);
    console.log("Get testnet AVAX from: https://faucet.avax.network/");
    process.exit(1);
  }

  const HTLC = await ethers.getContractFactory("HTLC");
  const htlc = await HTLC.deploy();

  await htlc.waitForDeployment();

  const address = await htlc.getAddress();
  console.log(" HTLC deployed to:", address);
  
  if (networkName === "fuji") {
    console.log(`Explorer: https://testnet.snowtrace.io/address/${address}`);
  } else if (networkName === "echo") {
    console.log(`Explorer: https://subnets-test.avax.network/echo/address/${address}`);
  }

  console.log("\nSave this address for your daemon configuration:");
  if (networkName === "fuji") {
    console.log(`HTLC_CCHAIN=${address}`);
  } else if (networkName === "echo") {
    console.log(`HTLC_SUBNET=${address}`);
  }

  console.log("\nTo verify on explorer:");
  console.log(`npx hardhat verify --network ${networkName} ${address}`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
