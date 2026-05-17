use alloy::sol;

sol! {
    #[derive(Debug)]
    contract HTLC {
        event SwapInitiated(bytes32 indexed hashlock, uint256 amount, address sender, uint256 timelock);
        event SwapClaimed(bytes32 indexed hashlock, bytes32 secret);
        function lock(uint256 amount, bytes32 hashlock, uint256 timelock) external payable;
        function claim(bytes32 secret) external;
    }
}

pub use HTLC::{SwapInitiated, SwapClaimed};