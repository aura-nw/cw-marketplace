use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid phase time")]
    InvalidPhaseTime {},

    #[error("Invalid phase id")]
    InvalidPhaseId {},

    #[error("Phase is inactivated")]
    PhaseIsInactivated {},

    #[error("Not enough funds")]
    NotEnoughFunds {},

    #[error("Max supply reached")]
    MaxSupplyReached {},

    #[error("User minted too much nfts")]
    UserMintedTooMuchNfts {},

    #[error("Launchpad started")]
    LaunchpadStarted {},

    #[error("Launchpad is already deactivated")]
    LaunchpadIsDeactivated {},

    #[error("Launchpad is already activated")]
    LaunchpadIsActivated {},

    #[error("Too many nfts")]
    TooManyNfts {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
