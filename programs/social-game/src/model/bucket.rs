use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Bucket {
    pub guaranteed_prize_per_bid: u64,
    pub rank_start: u32,
    pub rank_end: u32,
    pub weight_prize_per_bid: u64,
    pub bucket: u8
}

impl Bucket {
    pub const MAX_SIZE: usize = 8 + 4 + 4 + 8 + 1;
}

pub struct TempBucket {
    pub guaranteed_prize_per_bid: u64,
    pub guaranteed_prize: u64,
    pub rank_start: u32,
    pub rank_end: u32,
    pub weight: f64,
    pub total_weight: f64,
    pub participant_count: u32,
    pub weight_ratio: f64,
    pub weight_prize: u64,
    pub weight_prize_per_bid: u64,
    pub bucket: u8
}

impl TempBucket {
    pub fn get_weight_by_bucket(&self, bucket: u8) -> u32 {
        let max_weight = 100000;
        let max_weight_for_lower_buckets = 500;

        if bucket == 1 {
            max_weight
        } else if  bucket==2 { //2nd rank gets 10% of the first one
            max_weight/10
        } else if  bucket==3 { 
            (max_weight/10)*90/100
        } else if  bucket==4 { 
            (max_weight/10)*80/100
        } else if  bucket==5 { 
            (max_weight/10)*70/100
        } else if  bucket==6 { 
            (max_weight/10)*60/100
        } else if  bucket==7 { 
            (max_weight/10)*50/100
        } else if  bucket==8 { 
            (max_weight/10)*40/100
        } else if  bucket==9 { 
            (max_weight/10)*30/100
        } else if  bucket==10 { 
            (max_weight/10)*20/100
        } else if  bucket==11 { 
            (max_weight/10)*10/100
        } else { 
            max_weight_for_lower_buckets - 25*(bucket-12) as u32
        } 
    }
}