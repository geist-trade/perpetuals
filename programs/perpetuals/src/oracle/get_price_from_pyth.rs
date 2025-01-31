use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use switchboard_solana::AccountDeserialize;
use super::OraclePrice;
use crate::constants::*;
use crate::error::PerpetualsError;

#[inline(never)]
pub fn get_price_from_pyth(
    oracle_account: &AccountInfo,
    clock: &Clock,
    use_ema: bool
) -> Result<(OraclePrice, OraclePrice)> {
    let oracle_account_data = oracle_account.try_borrow_mut_data()?;
    
    let oracle: PriceUpdateV2 = PriceUpdateV2
        ::try_deserialize(&mut oracle_account_data.as_ref())
        .map_err(|_| PerpetualsError::PriceError)?;

    let price = oracle.get_price_no_older_than(
        &clock, 
        ORACLE_MAXIMUM_AGE, 
        &oracle.price_message.feed_id
    ).map_err(|_| PerpetualsError::PriceError)?;

    // if above succeeds, ema should work too
    let ema_price = oracle.price_message.ema_price;

    let exponent = price.exponent;

    Ok(
        OraclePrice {
            price: if (use_ema) { ema_price.try_into() } else { price.price.try_into() },
            exponent
        }
    )
}