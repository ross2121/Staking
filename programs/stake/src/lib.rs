use anchor_lang::system_program;
use anchor_spl::{associated_token::AssociatedToken, token::{MintTo, TokenAccount}};

use {
    anchor_lang::prelude::*,
    anchor_spl::{
        metadata::{
            create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
            CreateMetadataAccountsV3,Metadata
        },
        token::{Mint, Token, mint_to},
    },
};

// use crate::instruction::TokenProgram;
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
    use anchor_spl::token::{self, Burn};

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
    pub fn create_token_mint(
        ctx: Context<CreateToken>,
        _token_decimal: u8,
        token_name: String,
        token_symbol: String,
        token_uri: String
    ) -> Result<()> {
        msg!("Creating metadata for mint: {}", ctx.accounts.mint_account.key());
        msg!("Metadata account: {}", ctx.accounts.metadata_account.key());
        
        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.mint_account.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name: token_name,
                symbol: token_symbol,
                uri: token_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false, // is_mutable
            true,  // update_authority_is_signer
            None,  // collection
        )?;
        
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

        // Burn tokens from the associated token account
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint_account.to_account_info(),
                from: ctx.accounts.associated_token_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            }
        );
        token::burn(burn_ctx, amount * 10u64.pow(ctx.accounts.mint_account.decimals as u32))?;

        ctx.accounts.pda.staked_amount = ctx.accounts.pda.staked_amount
            .checked_sub(amount)
            .ok_or(StakeError::Unauthorized)?;
          
        Ok(())
    }
    pub fn mint_token(ctx: Context<Minttoken>, amount: u64) -> Result<()> {
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint_account.to_account_info(),
                    to: ctx.accounts.associated_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                }
            ),
            amount * 10u64.pow(ctx.accounts.mint_account.decimals as u32)
        )?;
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
       
        // let mint_ctx = Context::new(
        //     ctx.program_id,
        //     MintToken {
        //         mint_account: ctx.accounts.mint_account.clone(),
        //         associated_token_account: ctx.accounts.associated_token_account.clone(),
        //         token_program: ctx.accounts.token_program.clone(),
        //         payer: ctx.accounts.payer.clone(),
        //     },
        //     ctx.remaining_accounts,
        // );
        // mint_token(mint_ctx, amount)?;
        system_program::transfer(cpi_context, amount * LAMPORTS_PER_SOL)?;
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint_account.to_account_info(),
                    to: ctx.accounts.associated_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                }
            ),
            amount * 10u64.pow(ctx.accounts.mint_account.decimals as u32)
        )?;
        
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
pub struct  Createmint{
 pub name:String,
 pub url:String,
 pub symbol:String
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
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds=[b"client1", signer.key().as_ref()],
        bump=pda.bump,
        constraint = pda.owner == signer.key() @ StakeError::Unauthorized
    )]
    pub pda: Account<'info, StakeAccount>,
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = signer
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds=[b"client1", signer.key().as_ref()], 
        bump=pda.bump,
    )]
    pub pda: Account<'info, StakeAccount>,
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = signer
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    
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
#[derive(Accounts)]
#[instruction(_token_decimals: u8,token_name:String,token_symbol:String,token_uri:String)]
pub struct CreateToken<'info>{
#[account(mut)]
pub payer:Signer<'info>,

    /// CHECK: Validate address by deriving pda
#[account(mut,seeds=[b"metadata",token_metadata.key().as_ref(),mint_account.key().as_ref()],bump,seeds::program=token_metadata.key())]
pub metadata_account:UncheckedAccount<'info>,
#[account(init,payer=payer,mint::decimals=_token_decimals,mint::authority=payer.key())]
pub mint_account:Account<'info,Mint>,
pub token_metadata:Program<'info,Metadata>,
pub token_program:Program<'info,Token>,
pub system_program:Program<'info,System>,
pub rent:Sysvar<'info,Rent>
} 
#[derive(Accounts, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Minttoken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub recipent: SystemAccount<'info>,
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = recipent
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
#[account]
pub struct NewAccount {
    pub data: u32,
}
// pub struct Amount{

// }