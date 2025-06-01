use anchor_lang::prelude::*;
use anchor_lang::system_program;
declare_id!("EFJQgNqJMtZCBhHtsAhYm9zkj5nCozrsTsM6GZdo7uG9");
const POINTS: u64 = 1_000_000;
const LAMPORTS: u64 = 1_000_000;
const SECONDS: u64 = 86_400;

#[error_code]
pub enum StakeError {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Overflow occurred")]
    Overflow,
}

#[program]
pub mod stake {
 

    use anchor_lang::solana_program::{clock, native_token::LAMPORTS_PER_SOL};

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
    pub fn unstake(ctx: Context<Unstake>,amount: u64)-> Result<()>{
        require!(amount>0,StakeError::InvalidAmount);
        require!(ctx.accounts.pda.staked_amount>=amount,StakeError::InvalidAmount);
        let clock=Clock::get()?;
        let bump=ctx.accounts.pda.bump;
        let signer_key=ctx.accounts.signer.key();
        update_point(&mut ctx.accounts.pda,clock.unix_timestamp)?;
        let signer_seeds: &[&[&[u8]]] = &[&[b"client1", signer_key.as_ref(), &[bump]]];
        let cpi_Context=CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(), system_program::Transfer{from:ctx.accounts.pda.to_account_info(),to:ctx.accounts.signer.to_account_info()}, signer_seeds);
        system_program::transfer(cpi_Context,amount)?;
        ctx.accounts.pda.staked_amount=ctx.accounts.pda.staked_amount.checked_sub(amount).ok_or(StakeError::Unauthorized)?;
          
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);
        let clock = Clock::get()?;
      let cpi_context=CpiContext::new(ctx.accounts.system_program.to_account_info(),system_program::Transfer{
        from:ctx.accounts.signer.to_account_info(),
        to:ctx.accounts.pda.to_account_info()
      });
      system_program::transfer(cpi_context,amount*LAMPORTS_PER_SOL)?;
      
      let pda_account = &mut ctx.accounts.pda;

  pda_account.staked_amount=pda_account.staked_amount.checked_add(amount).ok_or(StakeError::Overflow)?;
      update_point(pda_account, clock.epoch as i64)?;
        pda_account.staked_amount = pda_account.staked_amount.checked_add(amount).ok_or(StakeError::Overflow)?;
        
        Ok(())
    }
}


fn update_point(pda_account: &mut StakeAccount, current_time: i64) -> Result<()> {
    let time = current_time
        .checked_sub(pda_account.last_update_amount)
        .ok_or(StakeError::InvalidTimestamp)? as u64;
    if time > 0 && pda_account.staked_amount > 0 {
        let new_points = calculate_point_earned(pda_account.staked_amount, time)?;
        pda_account.point = pda_account.point.checked_add(new_points).ok_or(StakeError::Overflow)?;
    }
    pda_account.last_update_amount = current_time;
    Ok(())
}

fn calculate_point_earned(staked: u64, time: u64) -> Result<u64> {
    let points = (staked as u128)
        .checked_mul(time as u128)
        .ok_or(StakeError::Overflow)?
        .checked_mul(POINTS as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(SECONDS as u128)
        .ok_or(StakeError::Overflow)?;
    Ok(points as u64)
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
    #[account(mut,seeds=[b"client1",signer.key().as_ref()],bump=pda.bump,constraint = pda.owner == signer.key() @ StakeError::Unauthorized)]
    pub pda: Account<'info, StakeAccount>,
    pub system_program:Program<'info,System>
}
#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub signer:Signer<'info>,
    #[account(mut,seeds=[b"client1",signer.key.as_ref()],bump=pda.bump,constraint=pda.owner==signer.key()@StakeError::Unauthorized)]
    pub  pda:Account<'info,StakeAccount>,
    pub system_program:Program<'info,System>
}

#[account]
pub struct NewAccount {
    pub data: u32,
}
