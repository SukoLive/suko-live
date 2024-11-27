use {
    anchor_lang::{ prelude::* },
    anchor_spl::{
        associated_token::{ AssociatedToken },
        token_interface::{ Token2022 },
    },
};
use solana_program::{program::invoke, system_instruction};
use num::pow;

use crate::model::*;
use crate::utils::*;

const PER_TRANS_LAMPORTS: u64 = 500;

#[inline(never)]
pub fn create_contest(
    ctx: Context<CreateContest>,
    contest_id: String,
    object_id: String,
    contest_name: String,
    symbol: String,
    uri: String,
    _contest_type: String,
    _sub_type: String,
    start_time: i64,
    end_time: i64,
    bid_end_time: i64,
    minimum_bid: u64,
    company_share: u32,
    max_participants: u32,
    _link: String
) -> Result<()> {
    let contest = &mut ctx.accounts.contest;
    contest.object_id = object_id.clone();
    contest.contest_id = contest_id.clone();
    contest.start_time = start_time;
    contest.end_time = end_time;
    contest.bidding_end_time = bid_end_time;
    contest.status = ContestStatus::Active;
    contest.total_pot = 0;
    contest.company_share = company_share;
    contest.authority = *ctx.accounts.payer.key;
    contest.actual_count = 0;
    contest.nft_address = ctx.accounts.mint_account.key();
    contest.max_participants = max_participants;
    contest.minimum_bid = minimum_bid;
    contest.total_bids = 0;

    // This is the space required for the metadata account.
    // We put the meta data into the mint account at the end so we
    // don't need to create any additional account.
    let meta_data_space = 500;

    // We use a PDA as a mint authority for the metadata account because
    // we want to be able to update the NFT from the program.
    let seeds = b"nft_authority";
    let bump = ctx.bumps.nft_authority;
    let signer: &[&[&[u8]]] = &[&[seeds, &[bump]]];

    nft::mint(meta_data_space, contest_name.clone(), symbol, uri, signer, &ctx.accounts.token_program, &ctx.accounts.payer,
        &ctx.accounts.mint_account, &ctx.accounts.associated_token_program, &ctx.accounts.associated_token_account,
        &ctx.accounts.system_program, &ctx.accounts.nft_authority)?;

    nft::update_field("status".to_string(), "Active".to_string(), &ctx.accounts.mint_account, &ctx.accounts.nft_authority, signer)?;

    Ok(())
}

/*
#[inline(never)]
pub fn close_bidding(ctx: Context<CloseBidding>, _contest_id: String) -> Result<()> {
    let contest = &mut ctx.accounts.contest;
    contest.status = ContestStatus::BiddingClosed;
    Ok(())
}*/

#[inline(never)]
pub fn finalize_contest(ctx: Context<FinalizeContest>, _contest_id: String, actual_count: u32, result_url: String) -> Result<()> {
    let contest = &mut ctx.accounts.contest;
    require!(
        contest.status == ContestStatus::Active,
        ContestError::InvalidStatus
    );

    contest.status = ContestStatus::Completed;
    contest.actual_count = actual_count;

    if contest.total_pot > 0 {
        //Transfer house cut - which is the remaining amount.
        let house_cut = contest.total_pot - contest.total_paid - 10000;
        let transfer_instruction = system_instruction::transfer(ctx.accounts.contest_account.key, ctx.accounts.spydra_account.key, house_cut);
        invoke(
            &transfer_instruction,
            &[
                ctx.accounts.contest_account.to_account_info(),
                ctx.accounts.spydra_account.clone(),
                ctx.accounts.system_program.to_account_info(),
            ]
        )?;
    }

    let seeds = b"nft_authority";
    let bump = ctx.bumps.nft_authority;
    let signer: &[&[&[u8]]] = &[&[seeds, &[bump]]];
    nft::update_field("actual_count".to_string(), actual_count.to_string(), &ctx.accounts.mint_account, &ctx.accounts.nft_authority, signer)?;
    nft::update_field("results".to_string(), result_url, &ctx.accounts.mint_account, &ctx.accounts.nft_authority, signer)?;
    nft::update_field("status".to_string(), "Closed".to_string(), &ctx.accounts.mint_account, &ctx.accounts.nft_authority, signer)?;

    Ok(())
}

#[inline(never)]
pub fn create_buckets(ctx: Context<CreateBucket>, contest_id:String, total_winners: u32, max_rank: u32, company_share: u32) -> Result<()> {
    let contest = &mut ctx.accounts.contest;
    let bucket_account = &mut ctx.accounts.bucket_account;
    let payer = &ctx.accounts.payer;
    let mut bucket_count = 1;
    let mut power = 0;
    let mut rank_start = 1;
    let mut rank_end;
    let mut total_weight = 0.0;
    let total_pot = contest.total_pot;
    let house_cut = (company_share as u64  * total_pot)/10000;
     //subtract house cut from total pot
    let prize_for_bidders = total_pot-house_cut;

    //get the total bidder count which have ranks and then multiply with the contest min bid
    let total_guaranteed_prize = total_winners as u64 *contest.minimum_bid;
    //subtract gauranteed from the total prize for bidders
    let prize_left_after_guaranteed = prize_for_bidders-total_guaranteed_prize;
    
    let mut buckets = Vec::new();
    while rank_start <= max_rank {
        let mut bucket = TempBucket {
            guaranteed_prize_per_bid: contest.minimum_bid,
            rank_start: rank_start,
            bucket: bucket_count,
            guaranteed_prize: 0,
            rank_end: 0,
            weight: 0.0,
            total_weight: 0.0,
            participant_count: 0,
            weight_ratio: 0.0,
            weight_prize: 0,
            weight_prize_per_bid: 0
        };

        //Top 11 ranks will have single participant
        if bucket_count > 11 {
            power = power+1;
            rank_end = pow(2, power) + rank_start;
        }else{
            rank_end = rank_start;
        }

        if rank_end > max_rank {
            rank_end = max_rank;
        }

        bucket.rank_end = rank_end;
        bucket.participant_count = 1+rank_end-rank_start;
        bucket.weight = bucket.get_weight_by_bucket(bucket_count) as f64;
        //get weight for the bucket and multiply by total count for that rank
        bucket.total_weight = bucket.weight*bucket.participant_count as f64;
        total_weight = total_weight + bucket.total_weight;

        bucket.guaranteed_prize = bucket.guaranteed_prize_per_bid*bucket.participant_count as u64;
        buckets.push(bucket);
        bucket_count = bucket_count + 1;
        rank_start = rank_end + 1;
    } 

    /*
    //Initialize bucket PDA account.
    // Derive PDA address to ensure correctness
    let (pda, _pda_bump) = Pubkey::find_program_address(
        &[b"bucket".as_ref(), contest_id.as_bytes()],
        ctx.program_id,
    );

    // Ensure the derived PDA matches the provided PDA
    require!(pda == *bucket_account.key, ProgramErrorCode::InvalidBucketAccount);

    // Check if the account already exists
    if bucket_account.to_account_info().data_is_empty() {
        msg!("Creating Bucket PDA account...");
        // Allocate account space and fund it
        let space = 8 + ContestBucket::MAX_SIZE + Bucket::MAX_SIZE * buckets.len(); // Account size
        let lamports = Rent::get()?.minimum_balance(space);

        let create_account_ix = system_instruction::create_account(
            payer.key,
            bucket_account.key,
            lamports,
            space as u64,
            ctx.program_id,
        );

        // Invoke the system program to create the account
        anchor_lang::solana_program::program::invoke(
            &create_account_ix,
            &[
                payer.to_account_info(),
                bucket_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    let mut data = ContestBucket::try_from_slice(&bucket_account.to_account_info().data.borrow())?;
    data.buckets.clear();*/
    bucket_account.buckets.clear();

    for bucket in &mut buckets {
        //get the weight ratio for that bucket
        bucket.weight_ratio = bucket.total_weight/total_weight;
        bucket.weight_prize  = (bucket.weight_ratio*prize_left_after_guaranteed as f64) as u64;
        bucket.weight_prize_per_bid = bucket.weight_prize/bucket.participant_count as u64;
        msg!("weight, guaranteed_prize_per_bid, weight_prize_per_bid, bucket, rank_start, rank_end: {}, {}, {}, {}, {}, {}",
        bucket.weight, bucket.guaranteed_prize_per_bid, bucket.weight_prize_per_bid, bucket.bucket, bucket.rank_start, bucket.rank_end);
        
        let stored_bucket = Bucket{
            guaranteed_prize_per_bid: bucket.guaranteed_prize_per_bid,
            weight_prize_per_bid: bucket.weight_prize_per_bid,
            rank_start: bucket.rank_start,
            rank_end: bucket.rank_end,
            bucket: bucket.bucket
        };
        bucket_account.buckets.push(stored_bucket);
    }
    //data.serialize(&mut &mut bucket_account.to_account_info().data.borrow_mut()[..])?;
    
    Ok(())
}

pub fn distribute_winning(ctx: Context<DistributeWinning>, _contest_id:String, bid_id: String, rank: u32) -> Result<()> {
    let contest = &mut ctx.accounts.contest;
    let bucket_account = &ctx.accounts.bucket_account;
    let bid_account = &mut ctx.accounts.bid;

    //Check whether the amount has already been paid out.
    if bid_account.winning == 0 {
        if let Some(rank_bucket) = bucket_account.get_rank_bucket(rank) {
            let winning_amount = rank_bucket.guaranteed_prize_per_bid + rank_bucket.weight_prize_per_bid - PER_TRANS_LAMPORTS;

            //Transfer the winning amount
            let transfer_instruction = system_instruction::transfer(ctx.accounts.contest_account.key, &bid_account.user, winning_amount);
            invoke(
                &transfer_instruction,
                &[
                    ctx.accounts.contest_account.to_account_info(),
                    ctx.accounts.user_account.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ]
            )?;

            bid_account.rank = rank;
            bid_account.winning = winning_amount;
            contest.total_paid += winning_amount;

            msg!("{{ \"bid_id\": {}, \"user\": \"{}\", \"winning_amount\": {}, \"bucket\":{} }}", bid_id, bid_account.user, winning_amount, rank_bucket.bucket);
        }
    }
    
    //Deduct the cost for this transaction
    contest.total_paid += 5000;

    Ok(())
}

#[derive(Accounts)]
#[instruction(contest_id: String)]
pub struct CreateContest<'info> {
    #[account(init, seeds = [b"contest".as_ref(), contest_id.as_bytes()], bump, space = 8 + Contest::MAX_SIZE, payer = payer)]
    pub contest: Account<'info, Contest>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [b"admin_config".as_ref()], bump,
        constraint = admin_account.admin == payer.key()
    )]
    pub admin_account: Account<'info, AdminAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    #[account(mut)]
    pub mint_account: Signer<'info>,
    /// CHECK: We will create this one for the user
    #[account(mut)]
    pub associated_token_account: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(init_if_needed, seeds = [b"nft_authority".as_ref()], bump, space = 8, payer = payer)]
    pub nft_authority: Account<'info, NftAuthority>,
}

/*
#[derive(Accounts)]
#[instruction(_contest_id: String)]
pub struct CloseBidding<'info> {
    #[account(mut, seeds = [b"contest".as_ref(), _contest_id.as_bytes()], bump)]
    pub contest: Account<'info, Contest>,
    #[account(mut)]
    pub payer: Signer<'info>
    #[account(
        seeds = [b"admin_config".as_ref()], bump,
        constraint = admin_account.admin == payer.key()
    )]
    pub admin_account: Account<'info, AdminAccount>,
}*/

#[derive(Accounts)]
#[instruction(_contest_id: String)]
pub struct FinalizeContest<'info> {
    #[account(mut, seeds = [b"contest".as_ref(), _contest_id.as_bytes()], bump)]
    pub contest: Account<'info, Contest>,
    #[account(mut)]
    pub contest_account: Signer<'info>,
    #[account(mut)]
    pub mint_account: Signer<'info>,
    #[account(mut)]
    /// CHECK: Spydra wallet. This method can only be called by contest account as checked below
    pub spydra_account: AccountInfo<'info>,
    #[account(
        seeds = [b"admin_config".as_ref()], bump,
        has_one = contest_account
    )]
    pub admin_account: Account<'info, AdminAccount>,
    #[account(seeds = [b"nft_authority".as_ref()], bump)]
    pub nft_authority: Account<'info, NftAuthority>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(contest_id: String)]
pub struct CreateBucket<'info> {
    #[account(mut, seeds = [b"contest".as_ref(), contest_id.as_bytes()], bump)]
    pub contest: Account<'info, Contest>,
    #[account(init_if_needed, seeds = [b"bucket".as_ref(), contest_id.as_bytes()], bump, space = 8 + ContestBucket::MAX_SIZE + Bucket::MAX_SIZE * 30, payer = payer)]
    pub bucket_account: Account<'info, ContestBucket>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [b"admin_config".as_ref()], bump,
        constraint = admin_account.admin == payer.key()
    )]
    pub admin_account: Account<'info, AdminAccount>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(_contest_id: String, bid_id: String)]
pub struct DistributeWinning<'info> {
    #[account(mut, seeds = [b"contest".as_ref(), _contest_id.as_bytes()], bump)]
    pub contest: Account<'info, Contest>,
    #[account(seeds = [b"bucket".as_ref(), _contest_id.as_bytes()], bump)]
    pub bucket_account: Account<'info, ContestBucket>,
    #[account(mut, seeds = [b"bid".as_ref(), _contest_id.as_bytes(), bid_id.as_bytes()], bump)]
    pub bid: Account<'info, Bid>,
    #[account(mut)]
    pub contest_account: Signer<'info>,
    #[account(
        seeds = [b"admin_config".as_ref()], bump,
        has_one = contest_account
    )]
    pub admin_account: Account<'info, AdminAccount>,
    #[account(mut)]
    /// CHECK: User account to transfer funds to. This method can only be called by contest account as checked above
    pub user_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}