use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Already Exists")]
    AlreadyExists {},

    #[error("Insufficient Funds")]
    InsufficientFunds {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Nft not found")]
    NftNotFound {},

    #[error("Invalid end time")]
    InvalidEndTime {},
}
