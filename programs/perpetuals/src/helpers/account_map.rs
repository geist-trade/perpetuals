use anchor_lang::prelude::*;
use std::collections::BTreeMap;

use crate::error::PerpetualsError;

pub struct AccountMap {
    pub map: BTreeMap::<Pubkey, &AccountInfo>,
}

impl AccountMap {
    pub fn from_remaining_accounts(
        remaining_accounts: &[AccountInfo]
    ) -> Result<Self> {
        let map = BTreeMap::<Pubkey, &AccountInfo>::new();

        for i in 0..remaining_accounts.len() {
            let account = remaining_accounts[i];
            map.insert(account.key, &account);
        }

        Ok(AccountMap { map })
    }

    pub fn get_account(&self, key: &Pubkey) -> Result<&AccountInfo> {
        let account = self
            .map
            .get(key)
            .ok_or(PerpetualsError::AccountMapMissingEntry)?;

        Ok(account)
    }
}