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
 

    use anchor_lang::solana_program::{ clock, native_token::LAMPORTS_PER_SOL};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda;
        let clock = Clock::get()?;
        pda_account.owner = ctx.accounts.signer.key();
        pda_account.bump = ctx.bumps.pda;
        pda_account.point = 0;
        pda_account.staked_amount = 0;
        pda_account.last_update_amount = clock.epoch as i64;
        // let cpi_context = CpiContext::new(
        //     ctx.accounts.system_program.to_account_info(),
        //     system_program::Transfer {
        //         from: ctx.accounts.signer.to_account_info(),
        //         to: ctx.accounts.vault.to_account_info()
        //     }
        // );
        // system_program::transfer(cpi_context, 1 * LAMPORTS_PER_SOL)?;
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);
        require!(ctx.accounts.pda.staked_amount >= amount, StakeError::InvalidAmount);
        let clock = Clock::get()?;
        update_point(&mut ctx.accounts.pda, clock.unix_timestamp)?;
        let signer_key = ctx.accounts.signer.key();
        let vault_seeds = &[b"vault", signer_key.as_ref(), &[ctx.bumps.vault]];
        let signer_seeds = &[&vault_seeds[..]];
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.signer.to_account_info()
            }
        ).with_signer(signer_seeds);
        system_program::transfer(cpi_context, amount * LAMPORTS_PER_SOL)?;
        ctx.accounts.pda.staked_amount = ctx.accounts.pda.staked_amount
            .checked_sub(amount)
            .ok_or(StakeError::Unauthorized)?;
          
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);
        let clock = Clock::get()?;
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info(),
                to: ctx.accounts.vault.to_account_info()
            }
        );
        system_program::transfer(cpi_context, amount * LAMPORTS_PER_SOL)?;
        
        let pda_account = &mut ctx.accounts.pda;
        pda_account.staked_amount = pda_account.staked_amount.checked_add(amount).ok_or(StakeError::Overflow)?;
        
        Ok(())
    }
pub fn claim_points(ctx: Context<ClaimPoints>) -> Result<()> {
    let pda = &mut ctx.accounts.pda_account;
    let clock = Clock::get()?;
    update_point(pda, clock.unix_timestamp)?;
    let claimable_points = pda.point / 1_000_000;
    msg!("User has {} claimable points", claimable_points);
    pda.point = 0;
    Ok(())
}
pub fn get_points(ctx:Context<GetPoints>)-> Result<()>{
let pda_account=&mut ctx.accounts.pda_account;
let clock=Clock::get()?;
let time_elapsed=clock.unix_timestamp.checked_sub(pda_account.last_update_amount).ok_or(StakeError::Overflow)?;
let new_points=calculate_point_earned(pda_account.staked_amount, time_elapsed)?;
let current_total_points=pda_account.point.checked_add(new_points).ok_or(StakeError::Overflow)?;
msg!("Current points: {}, Staked amount: {} SOL", 
current_total_points / 1_000_000, 
pda_account.staked_amount / LAMPORTS_PER_SOL);

    Ok(())
}
    // pub fn clains_points(ctx:Context<>)
}


fn update_point(pda_account: &mut StakeAccount, current_time: i64) -> Result<()> {
    let time = current_time
        .checked_sub(pda_account.last_update_amount)
        .ok_or(StakeError::InvalidTimestamp)?;
    if time > 0 && pda_account.staked_amount > 0 {
        let new_points = calculate_point_earned(pda_account.staked_amount, time)?;
        pda_account.point = pda_account.point.checked_add(new_points).ok_or(StakeError::Overflow)?;
    }
    pda_account.last_update_amount = current_time;
    pda_account.last_update_amount=current_time; 
    dbg!("ds");
    Ok(())
}

fn calculate_point_earned(staked: u64, time: i64) -> Result<u64> {
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
pub struct aultAccount {
   
    pub staked_amount: u64,
 
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + 8 + 8 + 32 + 1 + 8,
        seeds = [b"client1", signer.key().as_ref()],
        bump
    )]
    pub pda: Account<'info, StakeAccount>,
    /// CHECK: This is a PDA that holds the SOL
    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds=[b"client1", signer.key().as_ref()],
        bump=pda.bump,
        constraint = pda.owner == signer.key() @ StakeError::Unauthorized
    )]
    pub pda: Account<'info, StakeAccount>,
    /// CHECK: This is a PDA that holds the SOL
    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds=[b"client1", signer.key().as_ref()], 
        bump=pda.bump,
    )]
    pub pda: Account<'info, StakeAccount>,
    /// CHECK: This is a PDA that holds the SOL
    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimPoints<'info>{
    #[account(mut)]
    pub user:Signer<'info>,
    #[account(mut,seeds=[b"client1",user.key().as_ref()],
     bump=pda_account.bump,
     constraint=pda_account.owner==user.key() @StakeError::Unauthorized
     )]
     pub pda_account:Account<'info,StakeAccount>
}

#[derive(Accounts)]
pub struct GetPoints<'info>{
    #[account(mut)]
    pub user:Signer<'info>,
    #[account(mut,seeds=[b"client1",user.key().as_ref()],
     bump=pda_account.bump,
     constraint=pda_account.owner==user.key() @StakeError::Unauthorized
     )]
     pub pda_account:Account<'info,StakeAccount>
}

#[account]
pub struct NewAccount {
    pub data: u32,
}
// pub struct Amount{

// }