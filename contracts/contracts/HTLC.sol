// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract HTLC {
    struct Swap { uint256 amount; address sender; uint256 timelock; bool claimed; }
    mapping(bytes32 => Swap) public swaps;

    event SwapInitiated(bytes32 indexed hashlock, uint256 amount, address sender, uint256 timelock);
    event SwapClaimed(bytes32 indexed hashlock, bytes32 secret);

    function lock(uint256 amount, bytes32 hashlock, uint256 timelock) external payable {
        require(msg.value == amount, "Incorrect amount");
        require(swaps[hashlock].amount == 0, "Hashlock used");
        swaps[hashlock] = Swap(amount, msg.sender, timelock, false);
        emit SwapInitiated(hashlock, amount, msg.sender, timelock);
    }

    function claim(bytes32 secret) external {
        bytes32 hashlock = keccak256(abi.encodePacked(secret));
        Swap storage s = swaps[hashlock];
        require(s.amount > 0 && !s.claimed && block.timestamp <= s.timelock);
        s.claimed = true;
        (bool success, ) = payable(msg.sender).call{value: s.amount}("");
        require(success, "Transfer failed");
        emit SwapClaimed(hashlock, secret);
    }
}