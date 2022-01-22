use cosmwasm_std::Addr;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Blacklisted address {addr:?}")]
    Blacklisted { addr: Addr },

    #[error("Game between these two players already exists")]
    DuplicateGame {},

    #[error("Game not found")]
    GameNotFound {},

    // this should never happen, it's just to exhaust arms in result match
    #[error("Game result not found")]
    GameResultNotFound {},

    #[error("Cannot finish the game")]
    CannotFinishGame {},

    #[error("Cannot start game against yourself")]
    GameAgainstYourself {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
