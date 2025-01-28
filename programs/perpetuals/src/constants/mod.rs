use anchor_lang::prelude::*;

#[constant]
pub const ORACLE_MAXIMUM_AGE: u64 = 60; // seconds, should be lowered in prod

#[constant]
pub const ADMIN_SEED: &str = "admin";

#[constant]
pub const PERPETUALS_SEED: &str = "perpetuals";

#[constant]
pub const POOL_SEED: &str = "pool";

#[constant]
pub const LP_TOKEN_MINT_SEED: &str = "lp_token_mint";