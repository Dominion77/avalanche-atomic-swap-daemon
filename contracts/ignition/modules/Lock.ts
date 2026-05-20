// This setup uses Hardhat Ignition to manage smart contract deployments.
// Learn more about it at https://hardhat.org/ignition

import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const HTLCModule = buildModule("HTLCModule", (m) => {
  const htlc = m.contract("HTLC");

  return { htlc };
});

export default HTLCModule;
