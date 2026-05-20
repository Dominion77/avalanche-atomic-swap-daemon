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
        #[allow(dead_code)]
        amount: U256,
        #[allow(dead_code)]
        timelock: u64,
    },
    #[allow(dead_code)]
    LockedOnBoth,
    #[allow(dead_code)]
    ClaimedOnMirror,
}