import { expect } from "chai";
import { ethers } from "hardhat";
import { HTLC } from "../typechain-types";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";

describe("HTLC", function () {
  let htlc: HTLC;
  let owner: HardhatEthersSigner;
  let user: HardhatEthersSigner;
  let secret: string;
  let hashlock: string;

  beforeEach(async function () {
    [owner, user] = await ethers.getSigners();

    const HTLCFactory = await ethers.getContractFactory("HTLC");
    const deployedContract = await HTLCFactory.deploy();
    await deployedContract.waitForDeployment();
    htlc = deployedContract as unknown as HTLC;

    // Generate secret and hashlock
    secret = ethers.id("my_secret_123");
    hashlock = ethers.keccak256(secret);
  });

  describe("Deployment", function () {
    it("Should deploy successfully", async function () {
      expect(await htlc.getAddress()).to.be.properAddress;
    });
  });

  describe("Lock", function () {
    it("Should lock funds with correct parameters", async function () {
      const amount = ethers.parseEther("1.0");
      const timelock = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now

      await expect(
        htlc.connect(user).lock(amount, hashlock, timelock, { value: amount })
      )
        .to.emit(htlc, "SwapInitiated")
        .withArgs(hashlock, amount, user.address, timelock);

      const swap = await htlc.swaps(hashlock);
      expect(swap.amount).to.equal(amount);
      expect(swap.sender).to.equal(user.address);
      expect(swap.timelock).to.equal(timelock);
      expect(swap.claimed).to.be.false;
    });

    it("Should reject if value doesn't match amount", async function () {
      const amount = ethers.parseEther("1.0");
      const wrongValue = ethers.parseEther("0.5");
      const timelock = Math.floor(Date.now() / 1000) + 3600;

      await expect(
        htlc.connect(user).lock(amount, hashlock, timelock, { value: wrongValue })
      ).to.be.revertedWith("Incorrect amount");
    });

    it("Should reject if hashlock already used", async function () {
      const amount = ethers.parseEther("1.0");
      const timelock = Math.floor(Date.now() / 1000) + 3600;

      await htlc.connect(user).lock(amount, hashlock, timelock, { value: amount });

      await expect(
        htlc.connect(user).lock(amount, hashlock, timelock, { value: amount })
      ).to.be.revertedWith("Hashlock used");
    });
  });

  describe("Claim", function () {
    beforeEach(async function () {
      const amount = ethers.parseEther("1.0");
      const timelock = Math.floor(Date.now() / 1000) + 3600;
      await htlc.connect(user).lock(amount, hashlock, timelock, { value: amount });
    });

    it("Should allow claim with correct secret", async function () {
      const balanceBefore = await ethers.provider.getBalance(owner.address);

      await expect(htlc.connect(owner).claim(secret))
        .to.emit(htlc, "SwapClaimed")
        .withArgs(hashlock, secret);

      const balanceAfter = await ethers.provider.getBalance(owner.address);
      expect(balanceAfter).to.be.gt(balanceBefore);

      const swap = await htlc.swaps(hashlock);
      expect(swap.claimed).to.be.true;
    });

    it("Should reject claim with wrong secret", async function () {
      const wrongSecret = ethers.id("wrong_secret");

      await expect(
        htlc.connect(owner).claim(wrongSecret)
      ).to.be.reverted;
    });

    it("Should reject double claim", async function () {
      await htlc.connect(owner).claim(secret);

      await expect(
        htlc.connect(owner).claim(secret)
      ).to.be.reverted;
    });

    it("Should reject claim after timelock expires", async function () {
      // Deploy new contract with expired timelock
      const amount = ethers.parseEther("1.0");
      const expiredTimelock = Math.floor(Date.now() / 1000) - 1; // Already expired
      const newSecret = ethers.id("new_secret");
      const newHashlock = ethers.keccak256(newSecret);

      await htlc.connect(user).lock(amount, newHashlock, expiredTimelock, { value: amount });

      await expect(
        htlc.connect(owner).claim(newSecret)
      ).to.be.reverted;
    });
  });

  describe("Full Swap Flow", function () {
    it("Should complete a full atomic swap", async function () {
      const amount = ethers.parseEther("0.5");
      const timelock = Math.floor(Date.now() / 1000) + 3600;

      // User locks funds
      await htlc.connect(user).lock(amount, hashlock, timelock, { value: amount });

      // Verify lock
      const swap = await htlc.swaps(hashlock);
      expect(swap.amount).to.equal(amount);
      expect(swap.claimed).to.be.false;

      // Owner claims with secret
      const ownerBalanceBefore = await ethers.provider.getBalance(owner.address);
      await htlc.connect(owner).claim(secret);

      // Verify claim
      const ownerBalanceAfter = await ethers.provider.getBalance(owner.address);
      expect(ownerBalanceAfter).to.be.gt(ownerBalanceBefore);

      const swapAfter = await htlc.swaps(hashlock);
      expect(swapAfter.claimed).to.be.true;
    });
  });
});
