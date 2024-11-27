use anchor_lang::{ prelude::* };
use solana_program::{program::invoke_signed, native_token::LAMPORTS_PER_SOL, system_instruction};
use anchor_spl::token_interface::Token2022;
use crate::model::*;
use crate::utils::*;

#[inline(never)]
pub fn place_bid(
    ctx: Context<PlaceBid>,
    contest_id: String,
    bid_id: String,
    predicted_count: u32,
) -> Result<()> {
    let contest = &mut ctx.accounts.contest;
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        contest.status == ContestStatus::Active,
        ContestError::ContestNotActive
    );
    require!(
        current_time <= contest.bidding_end_time,
        ContestError::BiddingClosed
    );

    // Transfer SOL from user to contest account
    let lamports = contest.minimum_bid;
    // Create the transfer instruction
    let transfer_instruction = system_instruction::transfer(ctx.accounts.user.key, ctx.accounts.contest_account.key, lamports);

    // Invoke the transfer instruction
    invoke_signed(
        &transfer_instruction,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.contest_account.clone(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[],
    )?;

    // Record the bid
    let bid = &mut ctx.accounts.bid;
    bid.user = *ctx.accounts.user.key;
    bid.predicted_count = predicted_count;

    msg!("Created bid: id - {}, wallet address - {}, predicted count - {} for contest id - {}", bid_id, bid.user, bid.predicted_count, contest_id);

    // Update total pot
    contest.total_pot += lamports;
    contest.total_bids += 1;
    // Update the NFT metadata account with total_pot
    let total_pot = contest.total_pot as f32 / LAMPORTS_PER_SOL as f32;
    let seeds = b"nft_authority";
    let bump = ctx.bumps.nft_authority;
    let signer: &[&[&[u8]]] = &[&[seeds, &[bump]]];

    nft::update_field("total_pot".to_string(), total_pot.to_string(), &ctx.accounts.mint_account, &ctx.accounts.nft_authority, signer)?;
    nft::update_field("total_bids".to_string(), contest.total_bids.to_string(), &ctx.accounts.mint_account, &ctx.accounts.nft_authority, signer)?;

    Ok(())
}

#[inline(never)]
pub fn edit_bid(
    ctx: Context<EditBid>,
    contest_id: String,
    bid_id: String,
    predicted_count: u32
) -> Result<()> {
    let contest = &mut ctx.accounts.contest;
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        contest.status == ContestStatus::Active,
        ContestError::ContestNotActive
    );
    require!(
        current_time <= contest.bidding_end_time,
        ContestError::BiddingClosed
    );

    // Record the bid
    let bid = &mut ctx.accounts.bid;
    bid.predicted_count = predicted_count;

    msg!("Edited bid: id - {}, wallet address - {}, predicted count - {} for contest id - {}", bid_id, bid.user, bid.predicted_count, contest_id);

    Ok(())
}

#[derive(Accounts)]
#[instruction(contest_id: String, bid_id: String)]
pub struct PlaceBid<'info> {
    #[account(mut, seeds = [b"contest".as_ref(), contest_id.as_bytes()], bump)]
    pub contest: Account<'info, Contest>,
    #[account(init, seeds = [b"bid".as_ref(), contest_id.as_bytes(), bid_id.as_bytes()], bump, space = 8 + Bid::MAX_SIZE, payer = user)]
    pub bid: Account<'info, Bid>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: Contest account to hold the funds. Explicit check is made for this below.
    pub contest_account: AccountInfo<'info>,
    #[account(mut)]
    pub mint_account: Signer<'info>,
    #[account(
        seeds = [b"admin_config".as_ref()], bump,
        has_one = contest_account
    )]
    pub admin_account: Account<'info, AdminAccount>,
    #[account(seeds = [b"nft_authority".as_ref()], bump)]
    pub nft_authority: Account<'info, NftAuthority>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(contest_id: String, bid_id: String)]
pub struct EditBid<'info> {
    #[account(mut, seeds = [b"contest".as_ref(), contest_id.as_bytes()], bump)]
    pub contest: Account<'info, Contest>,
    #[account(
        mut, seeds = [b"bid".as_ref(), contest_id.as_bytes(), bid_id.as_bytes()], bump,
        has_one = user
    )]
    pub bid: Account<'info, Bid>,
    #[account(mut)]
    pub user: Signer<'info>
}