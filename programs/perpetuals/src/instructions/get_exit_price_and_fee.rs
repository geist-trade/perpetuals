//! GetExitPriceAndFee instruction handler

use {
    crate::{
        constants::{CUSTODY_SEED, PERPETUALS_SEED, POOL_SEED, POSITION_SEED},
        oracle::OraclePrice,
        state::{
            custody::Custody,
            perpetuals::{Perpetuals, PriceAndFee},
            pool::Pool,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
#[instruction(
    args: GetExitPriceAndFeeParams
)]
pub struct GetExitPriceAndFee<'info> {
    #[account(
        seeds = [
            PERPETUALS_SEED.as_bytes()
        ],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        seeds = [
            POOL_SEED.as_bytes(),
            &args.pool_id.to_le_bytes()
        ],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        seeds = [
            POSITION_SEED.as_bytes(),
            position.owner.as_ref(),
            pool.key().as_ref(),
            custody.key().as_ref(),
            &[position.side as u8]
        ],
        bump = position.bump
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        seeds = [
            CUSTODY_SEED.as_bytes(),
            pool.key().as_ref(),
            custody.mint.as_ref()
        ],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = custody_oracle_account.key() == custody.oracle.key()
    )]
    pub custody_oracle_account: AccountInfo<'info>,

    #[account(
        seeds = [CUSTODY_SEED.as_bytes(),
                 pool.key().as_ref(),
                 collateral_custody.mint.as_ref()],
        bump = collateral_custody.bump
    )]
    pub collateral_custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = collateral_custody_oracle_account.key() == collateral_custody.oracle.key()
    )]
    pub collateral_custody_oracle_account: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetExitPriceAndFeeParams {
    pool_id: u64,
}

pub fn get_exit_price_and_fee(
    ctx: Context<GetExitPriceAndFee>,
    _params: &GetExitPriceAndFeeParams,
) -> Result<PriceAndFee> {
    // compute exit price and fee
    let position = &ctx.accounts.position;
    let pool = &ctx.accounts.pool;
    let custody = &ctx.accounts.custody;
    let collateral_custody = &ctx.accounts.collateral_custody;

    let clock = &Clock::get()?;

    // match custody.oracle. {
    //     Oracle::Pyth(_) => {
    //         todo!()
    //     }
    // }

    // TODO: Separate basic and EMA oracles flow to increase readability of the code.

    let token_price = OraclePrice::new_from_oracle(
        &ctx.accounts.custody_oracle_account.to_account_info(),
        clock,
        custody.oracle,
        false,
    )?;

    let token_ema_price = OraclePrice::new_from_oracle(
        &ctx.accounts.custody_oracle_account.to_account_info(),
        clock,
        custody.oracle,
        custody.pricing.use_ema,
    )?;

    let collateral_token_ema_price = OraclePrice::new_from_oracle(
        &ctx.accounts
            .collateral_custody_oracle_account
            .to_account_info(),
        clock,
        collateral_custody.oracle,
        collateral_custody.pricing.use_ema,
    )?;

    let price = pool.get_exit_price(&token_price, &token_ema_price, position.side, custody)?;

    let size = token_ema_price.get_token_amount(position.size_usd, custody.decimals)?;

    let mut fee = pool.get_exit_fee(size, custody)?;

    if position.side == Side::Short || custody.is_virtual {
        let fee_amount_usd = token_ema_price.get_asset_amount_usd(fee, custody.decimals)?;
        fee = collateral_token_ema_price
            .get_token_amount(fee_amount_usd, collateral_custody.decimals)?;
    }

    Ok(PriceAndFee { price, fee })
}
