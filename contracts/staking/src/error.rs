use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("OverflowError")]
    OverflowError {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("exceed amout")]
    ExceedAmout {},
    
    #[error("Missing Withdraw Collateral Hook")]
    MissingWithdrawCollateralHook {},
}
