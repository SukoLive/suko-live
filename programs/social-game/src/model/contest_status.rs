use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ContestStatus {
    Active,
    BiddingClosed,
    Completed,
}