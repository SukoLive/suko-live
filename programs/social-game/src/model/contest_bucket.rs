use anchor_lang::prelude::*;
use crate::model::Bucket;

#[account]
pub struct ContestBucket {
    pub contest_id: String,
    pub buckets: Vec<Bucket>,
}

impl ContestBucket {
    pub const MAX_SIZE: usize = 8;

    pub fn get_rank_bucket(&self, rank: u32) -> Option<Bucket> {
        for bucket in &self.buckets {
            if rank >= bucket.rank_start && rank <= bucket.rank_end {
                return Some(bucket.clone());
            }
        }
        None
    }
}