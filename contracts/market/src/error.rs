use cosmwasm_std::{StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("OverflowError")]
    Overflow{},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Borrow amount too high; Loan liability becomes greater than borrow limit: {0}")]
    BorrowExceedsLimit(u128),

    #[error("Must deposit initial funds {0}{0}")]
    InitialFundsNotDeposited(u128, String),

    #[error("Invalid reply ID")]
    InvalidReplyId {},

    #[error("Exceeds stable max borrow factor; borrow demand too high")]
    MaxBorrowFactorReached {},

    #[error("Invalid request: \"redeem stable\" message not included in request")]
    MissingRedeemStableHook {},

    #[error("Not enough stable available; borrow demand too high")]
    NoStableAvailable {},

    #[error("Deposit amount must be greater than 0")]
    ZeroDeposit{},

    #[error("Repay amount must be greater than 0")]
    ZeroRepay {},
}
