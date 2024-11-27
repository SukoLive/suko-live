use anchor_lang::prelude::*;
use crate::model::ContestStatus;

#[account]
pub struct Contest {
    pub object_id: String,
    pub contest_id: String,
    pub nft_address: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub bidding_end_time: i64,
    pub status: ContestStatus,
    pub max_participants: u32,
    pub minimum_bid: u64,
    pub actual_count: u32,
    pub total_pot: u64,
    pub company_share: u32,     //stored as *100
    pub authority: Pubkey,
    pub total_bids: u64,
    pub total_paid: u64
}

impl Contest {
    pub const MAX_SIZE: usize = 32 + 8 + 32 + 8 + 8 + 8 + 1 + 4 + 8 + 4 + 8 + 4 + 32 + 8 + 8;
}