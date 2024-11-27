
use anchor_lang::{ prelude::* };
use crate::model::*;

#[inline(never)]
pub fn initialize_admin(ctx: Context<InitializeAdmin>, admin: Pubkey) -> Result<()> {
    let admin_account = &mut ctx.accounts.admin_account;

    require!(admin_account.admin == Pubkey::default() || admin_account.admin == ctx.accounts.signer.key(), ProgramErrorCode::AdminNotAuthorized);
    
    admin_account.admin = admin;

    Ok(())
}

pub fn set_contest_account(ctx: Context<InitializeAdmin>, contest_account: Pubkey) -> Result<()> {
    let admin_account = &mut ctx.accounts.admin_account;    
    admin_account.contest_account = contest_account;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(init_if_needed, seeds = [b"admin_config".as_ref()], bump, payer = signer, space = 8 + AdminAccount::MAX_SIZE)]
    pub admin_account: Account<'info, AdminAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetContestAccount<'info> {
    #[account(
        seeds = [b"admin_config".as_ref()], bump,
        constraint = admin_account.admin == signer.key()
    )]
    pub admin_account: Account<'info, AdminAccount>,

    #[account(mut)]
    pub signer: Signer<'info>
}
