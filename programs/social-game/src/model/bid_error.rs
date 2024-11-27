use anchor_lang::prelude::*;

#[error_code]
pub enum BidError {
    #[msg("Cannot retrieve bid account.")]
    BidAccountNotFound,
    #[msg("Cannot retrieve bid user's account.")]
    UserAccountNotFound
}   