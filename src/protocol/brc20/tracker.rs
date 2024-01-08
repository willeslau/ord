//! The tracking of each user's balance

use crate::protocol::brc20::balance::Balance;
use crate::protocol::brc20::storage::UserBalanceTable;
use crate::protocol::brc20::types::{Amount, Deploy, Mint, TokenId, Transfer};
use crate::protocol::brc20::{
  Error, InscriptionIdKey, TokenBalanceTable, TransferTable, UserBalanceKey,
};
use crate::InscriptionId;
use bitcoin::Address;
use redb::ReadableTable;

/*

 pub fn store_inscription(&mut self, id: InscriptionId, inscription: Inscription) -> Result<()> {
   let key = InscriptionIdKey { inner: id };
   let val = InscriptionValue { inner: inscription };
   self.inscription.insert(key, val)?;
   Ok(())
 }

 pub fn get_inscription_by_id(&mut self, id: InscriptionId) -> Result<Option<Inscription>> {
   let key = InscriptionIdKey { inner: id };
   Ok(self.inscription.get(key)?.map(|a| a.value().inner))
 }
*/
pub struct Tracker<'a, 'db, 'tx> {
  user_balances_table: &'a mut UserBalanceTable<'db, 'tx>,
  token_balance_table: &'a mut TokenBalanceTable<'db, 'tx>,
  token_transfer_table: &'a mut TransferTable<'db, 'tx>,
}

impl<'a, 'db, 'tx> Tracker<'a, 'db, 'tx> {
  pub fn new(
    user_balances_table: &'a mut UserBalanceTable<'db, 'tx>,
    token_balance_table: &'a mut TokenBalanceTable<'db, 'tx>,
    token_transfer_table: &'a mut TransferTable<'db, 'tx>,
  ) -> Self {
    Self {
      user_balances_table,
      token_balance_table,
      token_transfer_table,
    }
  }
}

impl<'a, 'db, 'tx> Tracker<'a, 'db, 'tx> {
  pub(crate) fn deploy(&mut self, _owner: &Address, payload: Deploy) -> Result<(), Error> {
    log::info!("deploy new token: {}", payload.token_id);

    if self.token_exists(&payload.token_id)? {
      return Err(Error::DuplicatedTokenDeployment(payload.token_id.clone()));
    }

    self
      .token_balance_table
      .insert(payload.token_id.clone(), Balance::new(Some(payload.max)))?;

    log::info!("new token written in db: {}", payload.token_id);

    Ok(())
  }

  pub(crate) fn mint(&mut self, owner: &Address, payload: Mint) -> Result<(), Error> {
    log::debug!("mint token to inscription owner: {}", owner);

    self.standard_check(&payload.token_id)?;

    let key = UserBalanceKey {
      token: payload.token_id.clone(),
      owner: owner.to_string(),
    };
    self.mint_inner(&key, payload.amount)?;

    log::info!("minted token to owner {owner} in cache");

    Ok(())
  }

  pub(crate) fn transfer(
    &mut self,
    from: &Address,
    to: &Address,
    inscription_id: InscriptionId,
  ) -> Result<(), Error> {
    let inscription_key = InscriptionIdKey { inner: inscription_id };
    let transfer = if let Some(transfer) = self.token_transfer_table.get(&inscription_key)? {
      transfer.value()
    } else {
      log::error!("transfer not found by id: {}, ignore", inscription_id);
      return Ok(());
    };

    let token_id = transfer.token_id;
    let amt = transfer.amount;

    self.transfer_inner(
      UserBalanceKey {
        token: token_id.clone(),
        owner: from.to_string(),
      },
      UserBalanceKey {
        token: token_id,
        owner: to.to_string(),
      },
      amt,
    )?;
    self.token_transfer_table.remove(&inscription_key)?;

    Ok(())
  }

  pub(crate) fn inscribe_transfer(
    &mut self,
    owner: &Address,
    inscription_id: InscriptionId,
    payload: Transfer,
  ) -> Result<(), Error> {
    log::debug!("owner {} inscribe transfer token {:?}", owner, payload);

    self.standard_check(&payload.token_id)?;

    let key = UserBalanceKey {
      token: payload.token_id.clone(),
      owner: owner.to_string(),
    };
    self.inscribe_transfer_inner(&key, payload.amount)?;
    self.token_transfer_table.insert(
      InscriptionIdKey {
        inner: inscription_id,
      },
      payload,
    )?;

    log::info!("burned token to address in cache");

    Ok(())
  }
}

impl<'a, 'db, 'tx> Tracker<'a, 'db, 'tx> {
  fn standard_check(&self, token_id: &TokenId) -> Result<(), Error> {
    if !self.token_exists(token_id)? {
      return Err(Error::TokenNotExists(token_id.clone()));
    }
    Ok(())
  }

  fn mint_inner(&mut self, key: &UserBalanceKey, amount: Amount) -> Result<(), Error> {
    let mut user_balance = self.get_user_balance(key)?;
    user_balance.incr_total(amount)?;

    let mut token_balance = self.get_token_balance(&key.token)?;
    token_balance.incr_total(amount)?;

    log::debug!(
      "user {} balance {:?} for token {}, token balance {:?}",
      key.owner,
      user_balance,
      key.token,
      token_balance
    );

    self.update_user_balance(key, user_balance)?;
    self.update_token_balance(&key.token, token_balance)
  }

  fn transfer_inner(
    &mut self,
    from: UserBalanceKey,
    to: UserBalanceKey,
    amount: Amount,
  ) -> Result<(), Error> {
    let mut from_balance = self.get_user_balance(&from)?;
    from_balance.decr_transferable(amount)?;
    from_balance.decr_total(amount)?;

    let mut to_balance = self.get_user_balance(&to)?;
    to_balance.incr_total(amount)?;

    self.update_user_balance(&from, from_balance)?;
    self.update_user_balance(&to, to_balance)
  }

  fn inscribe_transfer_inner(&mut self, key: &UserBalanceKey, amount: Amount) -> Result<(), Error> {
    let mut user_balance = self.get_user_balance(key)?;
    user_balance.incr_transferable(amount)?;

    log::debug!(
      "user {} balance {:?} for token {}",
      key.owner,
      user_balance,
      key.token
    );

    self.update_user_balance(key, user_balance)
  }

  fn update_user_balance(&mut self, key: &UserBalanceKey, balance: Balance) -> Result<(), Error> {
    self.user_balances_table.insert(key, balance)?;
    Ok(())
  }

  fn update_token_balance(&mut self, token_id: &TokenId, balance: Balance) -> Result<(), Error> {
    self.token_balance_table.insert(token_id.clone(), balance)?;
    Ok(())
  }

  /// Get the token balance, this method assumes the token already exists
  fn get_token_balance(&self, token_id: &TokenId) -> Result<Balance, Error> {
    Ok(
      self
        .token_balance_table
        .get(token_id)?
        .map(|i| i.value())
        .unwrap(),
    )
  }

  fn token_exists(&self, token_id: &TokenId) -> Result<bool, Error> {
    Ok(self.token_balance_table.get(token_id)?.is_some())
  }

  fn get_user_balance(&self, key: &UserBalanceKey) -> Result<Balance, Error> {
    Ok(
      self
        .user_balances_table
        .get(key)?
        .map(|v| v.value())
        .unwrap_or_default(),
    )
  }
}
