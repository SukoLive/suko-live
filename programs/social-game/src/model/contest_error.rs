use anchor_lang::prelude::*;

#[error_code]
pub enum ContestError {
    #[msg("Contest is not active.")]
    ContestNotActive,
    #[msg("Bidding period is closed.")]
    BiddingClosed,
    #[msg("Invalid contest status.")]
    InvalidStatus,
    #[msg("Cant initialize bucket account")]
    InvalidBucketAccount,
}   