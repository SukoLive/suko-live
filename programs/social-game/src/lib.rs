#![allow(clippy::result_large_err)]

pub mod instructions;
pub mod utils;
pub mod model;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("Gczm5pMWsEgEW8NFHoq54whvzmfja9gaimRzZtxTizfH");

#[program]
pub mod social_game_sol {
    use super::*;

    pub fn create_contest(
        ctx: Context<CreateContest>,
        contest_id: String,
        object_id: String,
        contest_name: String,
        symbol: String,
        uri: String,
        contest_type: String,
        sub_type: String,
        start_time: i64,
        end_time: i64,
        bid_end_time: i64,
        minimum_bid: u64,
        company_share: u32,
        max_participants: u32,
        link: String
    ) -> Result<()> {

        contest::create_contest(ctx, contest_id, object_id, contest_name, symbol, uri, contest_type, sub_type, 
        start_time, end_time, bid_end_time, minimum_bid, company_share, max_participants, link)
    }

    pub fn place_bid(
        ctx: Context<PlaceBid>,
        contest_id: String,
        bid_id: String,
        predicted_count: u32,
    ) -> Result<()> {
        bid::place_bid(ctx, contest_id, bid_id, predicted_count)
    }

    pub fn edit_bid(
        ctx: Context<EditBid>,
        contest_id: String,
        bid_id: String,
        predicted_count: u32,
    ) -> Result<()> {
        bid::edit_bid(ctx, contest_id, bid_id, predicted_count)
    }

    /*pub fn close_bidding(ctx: Context<CloseBidding>, contest_id: String) -> Result<()> {
        contest::close_bidding(ctx, contest_id)
    }*/

    pub fn finalize_contest(ctx: Context<FinalizeContest>, contest_id: String, actual_count: u32, result_url: String) -> Result<()> {
        contest::finalize_contest(ctx, contest_id, actual_count, result_url)
    }

    pub fn create_buckets(ctx: Context<CreateBucket>, contest_id: String, total_winners: u32, max_rank: u32, company_share: u32) -> Result<()> {
        contest::create_buckets(ctx, contest_id, total_winners, max_rank, company_share)
    }

    pub fn distribute_winning(ctx: Context<DistributeWinning>, contest_id:String, bid_id: String, rank: u32) -> Result<()> {
        contest::distribute_winning(ctx, contest_id, bid_id, rank)
    }

    pub fn initialize_admin(ctx: Context<InitializeAdmin>, admin: Pubkey) -> Result<()> {
        admin::initialize_admin(ctx, admin)
    }

    pub fn set_contest_account(ctx: Context<InitializeAdmin>, contest_account: Pubkey) -> Result<()> {
        admin::set_contest_account(ctx, contest_account)
    }
}