use anchor_lang::prelude::*;
use std::collections::BTreeMap;

use crate::error::PerpetualsError;

pub struct AccountMap<'a> {
    pub map: BTreeMap::<Pubkey, &'a AccountInfo<'a>>,
}

impl<'a> AccountMap<'a> {
    pub fn from_remaining_accounts(
        remaining_accounts: &'a [AccountInfo<'a>]
    ) -> Self {
        let mut map = BTreeMap::new();

        for account in remaining_accounts {
            map.insert(*account.key, account);
        }

        AccountMap { map }
    }

    pub fn get_account(&self, key: &Pubkey) -> Result<&AccountInfo<'a>> {
        self.map
            .get(key)
            .copied()
            .ok_or(PerpetualsError::AccountMapMissingEntry.into())
    }
}