//! AddPool instruction handler
use crate::constants::{PERPETUALS_SEED, LP_TOKEN_MINT_SEED, POOL_SEED};
use {
    crate::{
        error::PerpetualsError,
        state::{
            multisig::{AdminInstruction, Multisig},
            perpetuals::Perpetuals,
            pool::Pool,
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token},
};

#[derive(Accounts)]
#[instruction(params: AddPoolParams)]
pub struct AddPool<'info> {
    // Allow anyone to add new pool.
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERPETUALS_SEED.as_bytes()
        ],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        init,
        payer = signer,
        space = Pool::LEN,
        seeds = [
            POOL_SEED.as_bytes(),
            &perpetuals.pools.to_le_bytes(),
        ],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        payer = signer,
        mint::authority = pool,
        mint::freeze_authority = pool,
        mint::decimals = Perpetuals::LP_DECIMALS,
        seeds = [
            LP_TOKEN_MINT_SEED.as_bytes(),
            pool.key().as_ref()
        ],
        bump
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddPoolParams {
    pub name: String,
}

pub fn add_pool<'info>(
    ctx: Context<'_, '_, '_, 'info, AddPool<'info>>,
    params: &AddPoolParams,
) -> Result<u8> {
    // record pool data
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let pool = ctx.accounts.pool.as_mut();
    
    pool.inception_time = perpetuals.get_time()?;
    pool.name = params.name.clone();
    pool.bump = *ctx.bumps.get("pool").ok_or(ProgramError::InvalidSeeds)?;
    pool.lp_token_bump = *ctx
        .bumps
        .get("lp_token_mint")
        .ok_or(ProgramError::InvalidSeeds)?;

    // TODO: Inspect what this is doing under the hood.
    if !pool.validate() {
        return err!(PerpetualsError::InvalidPoolConfig);
    }

    perpetuals.pools += 1;

    // TODO: Add event for off-chain indexing.

    Ok(0)
}
