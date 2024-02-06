use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("VAura address not set")]
    VauraAddressNotSet {},

    #[error("Offer token allowance insufficient")]
    InsufficientAllowance {},

    #[error("Invalid token address")]
    InvalidTokenAddress {},
}
