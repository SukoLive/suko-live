use anchor_lang::{ prelude::* };

#[account]
pub struct AdminAccount {
    pub admin: Pubkey,      // Public key of the admin
    pub contest_account: Pubkey
}

impl AdminAccount {
    pub const MAX_SIZE: usize = 32 + 32;
}