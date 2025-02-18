//! GetLiquidationState instruction handler

use {
    crate::{
        constants::{CUSTODY_SEED, PERPETUALS_SEED, POOL_SEED, POSITION_SEED},
        oracle::OraclePrice,
        state::{custody::Custody, perpetuals::Perpetuals, pool::Pool, position::Position},
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct GetLiquidationState<'info> {
    #[account(
        seeds = [PERPETUALS_SEED.as_bytes()],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        seeds = [POOL_SEED.as_bytes(),
                 pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        seeds = [POSITION_SEED.as_bytes(),
                 position.owner.as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
                 &[position.side as u8]],
        bump = position.bump
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        seeds = [CUSTODY_SEED.as_bytes(),
                 pool.key().as_ref(),
                 custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = custody_oracle_account.key() == custody.oracle.key()
    )]
    pub custody_oracle_account: AccountInfo<'info>,

    #[account(
        constraint = position.collateral_custody == collateral_custody.key()
    )]
    pub collateral_custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = collateral_custody_oracle_account.key() == collateral_custody.oracle.key()
    )]
    pub collateral_custody_oracle_account: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetLiquidationStateParams {}

pub fn get_liquidation_state(
    ctx: Context<GetLiquidationState>,
    _params: &GetLiquidationStateParams,
) -> Result<u8> {
    let custody = &ctx.accounts.custody;
    let collateral_custody = &ctx.accounts.collateral_custody;
    let curtime = ctx.accounts.perpetuals.get_time()?;
    let clock = Clock::get()?;

    let token_price = OraclePrice::new_from_oracle(
        &ctx.accounts.custody_oracle_account.to_account_info(),
        &clock,
        custody.oracle,
        false,
    )?;

    let token_ema_price = OraclePrice::new_from_oracle(
        &ctx.accounts.custody_oracle_account.to_account_info(),
        &clock,
        custody.oracle,
        custody.pricing.use_ema,
    )?;

    let collateral_token_price = OraclePrice::new_from_oracle(
        &ctx.accounts
            .collateral_custody_oracle_account
            .to_account_info(),
        &clock,
        collateral_custody.oracle,
        false,
    )?;

    let collateral_token_ema_price = OraclePrice::new_from_oracle(
        &ctx.accounts
            .collateral_custody_oracle_account
            .to_account_info(),
        &clock,
        collateral_custody.oracle,
        collateral_custody.pricing.use_ema,
    )?;

    if ctx.accounts.pool.check_leverage(
        &ctx.accounts.position,
        &token_price,
        &token_ema_price,
        custody,
        &collateral_token_price,
        &collateral_token_ema_price,
        collateral_custody,
        curtime,
        false,
    )? {
        Ok(0)
    } else {
        Ok(1)
    }
}
