use anchor_lang::prelude::*;

declare_id!("EFJQgNqJMtZCBhHtsAhYm9zkj5nCozrsTsM6GZdo7uG9");

#[program]
pub mod stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda;
        let clock = Clock::get()?;
        pda_account.owner = ctx.accounts.signer.key();
        pda_account.bump = ctx.bumps.pda;
        pda_account.point = 0;
        pda_account.staked_amount = 0;
        pda_account.last_update_amount = clock.epoch as i64;

        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[account]
pub struct StakeAccount {
    pub point: u64,
    pub staked_amount: u64,
    pub owner: Pubkey,
    pub bump: u8,
    pub last_update_amount: i64
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + 8 + 8 + 32 + 1 + 8, // discriminator + point + staked_amount + owner + bump + last_update_amount
        seeds = [b"client1", signer.key().as_ref()],
        bump
    )]
    pub pda: Account<'info, StakeAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub pda: Account<'info, StakeAccount>
}

#[account]
pub struct NewAccount {
    pub data: u32,
}
