use alloy::primitives::U256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwapDirection {
    CToS,
    SToC,
}

#[derive(Debug, Clone)]
pub enum SwapState {
    Initiated {
        direction: SwapDirection,
        amount: U256,
        timelock: u64,
    },
    LockedOnBoth,
    ClaimedOnMirror,
}