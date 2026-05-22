use alloy::primitives::U256;

/// Direction of an atomic swap
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwapDirection {
    /// C-Chain to Subnet swap
    CToS,
    /// Subnet to C-Chain swap
    SToC,
}

/// State of an atomic swap
#[derive(Debug, Clone)]
pub enum SwapState {
    /// Swap has been initiated on one chain
    Initiated {
        /// Direction of the swap
        direction: SwapDirection,
        #[allow(dead_code)]
        /// Amount being swapped
        amount: U256,
        #[allow(dead_code)]
        /// Timelock expiration timestamp
        timelock: u64,
    },
    #[allow(dead_code)]
    /// Swap has been locked on both chains
    LockedOnBoth,
    #[allow(dead_code)]
    /// Swap has been claimed on the mirror chain
    ClaimedOnMirror,
}