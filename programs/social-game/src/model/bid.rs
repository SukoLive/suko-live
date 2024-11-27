use anchor_lang::prelude::*;

#[account]
pub struct Bid {
    pub user: Pubkey,
    pub predicted_count: u32,
    pub rank: u32,
    pub winning: u64
}

impl Bid {
    pub const MAX_SIZE: usize = 32 + 4 + 4 + 8;
}